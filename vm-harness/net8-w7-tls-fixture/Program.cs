using System.Net;
using System.Net.Security;
using System.Net.Sockets;
using System.Security.Authentication;
using System.Security.Cryptography;
using System.Security.Cryptography.X509Certificates;
using System.Text;
using System.Text.Json;

if (args.Length != 4)
    throw new ArgumentException("usage: <artifact> <ready-json> <result-json> <mode-file>");

var artifactPath = Path.GetFullPath(args[0]);
var readyPath = Path.GetFullPath(args[1]);
var resultPath = Path.GetFullPath(args[2]);
var modePath = Path.GetFullPath(args[3]);
var artifact = File.ReadAllBytes(artifactPath);
var artifactSha256 = Convert.ToHexString(SHA256.HashData(artifact)).ToLowerInvariant();
var requestPath = $"/raios/cas/sha256/{artifactSha256}";
var expectedRequest = $"GET {requestPath} HTTP/1.1\r\nHost: w7.test.raios\r\nAccept: application/octet-stream\r\nConnection: close\r\n\r\n";

using var key = ECDsa.Create(ECCurve.NamedCurves.nistP256);
var certificateRequest = new CertificateRequest("CN=w7.test.raios", key, HashAlgorithmName.SHA256);
var san = new SubjectAlternativeNameBuilder();
san.AddDnsName("w7.test.raios");
certificateRequest.CertificateExtensions.Add(san.Build());
certificateRequest.CertificateExtensions.Add(new X509KeyUsageExtension(X509KeyUsageFlags.DigitalSignature, true));
certificateRequest.CertificateExtensions.Add(new X509EnhancedKeyUsageExtension(
    new OidCollection { new("1.3.6.1.5.5.7.3.1") }, true));
using var certificate = certificateRequest.CreateSelfSigned(
    DateTimeOffset.UtcNow.AddMinutes(-5), DateTimeOffset.UtcNow.AddHours(1));
var pin = Convert.ToHexString(SHA256.HashData(key.ExportSubjectPublicKeyInfo())).ToLowerInvariant();

var listener = new TcpListener(IPAddress.Loopback, 8443);
listener.Start(1);
AtomicJson(readyPath, new {
    schema = "raios.net8_w7_tls_fixture_ready.v0",
    sni = "w7.test.raios",
    host = "127.0.0.1",
    host_port = 8443,
    path = requestPath,
    artifact_length = artifact.Length,
    artifact_sha256 = artifactSha256,
    spki_sha256 = pin,
    process_id = Environment.ProcessId,
    run_id = Guid.NewGuid().ToString("N")
});

var connection = 0;
while (true)
{
    using var client = await listener.AcceptTcpClientAsync();
    connection++;
    var mode = File.Exists(modePath) ? File.ReadAllText(modePath).Trim() : "serve";
    if (mode == "silent")
    {
        using var stream = client.GetStream();
        var drain = new byte[4096];
        while (await stream.ReadAsync(drain) != 0) { }
        AtomicJson(resultPath, new {
            schema = "raios.net8_w7_tls_fixture_result.v0",
            connection, mode, phase = "after_tcp_accept_before_tls", client_closed = true
        });
        continue;
    }

    using var ssl = new SslStream(client.GetStream(), false);
    await ssl.AuthenticateAsServerAsync(new SslServerAuthenticationOptions {
        ServerCertificate = certificate,
        ClientCertificateRequired = false,
        EnabledSslProtocols = SslProtocols.Tls13,
        CertificateRevocationCheckMode = X509RevocationMode.NoCheck,
        AllowRenegotiation = false,
        AllowTlsResume = false
    });
    if (ssl.SslProtocol != SslProtocols.Tls13 ||
        ssl.NegotiatedCipherSuite != TlsCipherSuite.TLS_AES_128_GCM_SHA256)
        throw new AuthenticationException($"unexpected TLS profile {ssl.SslProtocol}/{ssl.NegotiatedCipherSuite}");

    var requestBytes = new List<byte>(512);
    var one = new byte[1];
    while (requestBytes.Count < 4096 && await ssl.ReadAsync(one) == 1)
    {
        requestBytes.Add(one[0]);
        if (requestBytes.Count >= 4 &&
            requestBytes[requestBytes.Count - 4] == (byte)'\r' &&
            requestBytes[requestBytes.Count - 3] == (byte)'\n' &&
            requestBytes[requestBytes.Count - 2] == (byte)'\r' &&
            requestBytes[requestBytes.Count - 1] == (byte)'\n')
            break;
    }
    var requestText = Encoding.ASCII.GetString(requestBytes.ToArray());
    var exact = requestText == expectedRequest;
    if (!exact)
        throw new InvalidDataException("guest request did not match the fixed GET");

    var head = mode == "malformed"
        ? "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        : $"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {artifact.Length}\r\nConnection: close\r\n\r\n";
    await ssl.WriteAsync(Encoding.ASCII.GetBytes(head));
    if (mode != "malformed")
        await ssl.WriteAsync(artifact);
    await ssl.FlushAsync();
    AtomicJson(resultPath, new {
        schema = "raios.net8_w7_tls_fixture_result.v0",
        connection, mode, tls_protocol = ssl.SslProtocol.ToString(),
        cipher_suite = ssl.NegotiatedCipherSuite.ToString(), request_exact = exact,
        request_path = requestPath, response_status = 200,
        response_content_type = mode == "malformed" ? "text/plain" : "application/octet-stream",
        response_content_length = mode == "malformed" ? 0 : artifact.Length
    });
}

static void AtomicJson(string path, object value)
{
    Directory.CreateDirectory(Path.GetDirectoryName(path)!);
    var temporary = path + "." + Guid.NewGuid().ToString("N") + ".tmp";
    File.WriteAllText(temporary, JsonSerializer.Serialize(value));
    File.Move(temporary, path, true);
}
