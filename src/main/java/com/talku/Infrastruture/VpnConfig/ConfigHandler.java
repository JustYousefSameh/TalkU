package com.talku.Infrastruture.VpnConfig;

import java.io.BufferedReader;
import java.io.FileNotFoundException;
import java.io.InputStreamReader;
import java.io.PrintWriter;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpRequest.BodyPublishers;
import java.net.http.HttpResponse;

import com.fasterxml.jackson.core.JsonParser.Feature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.talku.Controller.TalkUController.VCException;
import com.talku.Infrastruture.VpnConfig.ConfigModels.ServerConfig;
import com.talku.Infrastruture.Wireguard.WireguardTunnel;
import com.talku.Utils.PathHelpers;

import io.vavr.Tuple;
import io.vavr.Tuple2;
import io.vavr.control.Either;

public class ConfigHandler {

    public static void main(String[] args) {
        getConfigAndWriteToFile();
    }

    private static String apiURL = "https://talku.ddns.net:8000/exchangekeys/";

    /*
     * API key for the server. This should be kept secret and not hardcoded in the
     * code. In a real application, you would want to store this securely and
     * retrieve it at runtime.
     */

    private static String apiKey = "z~WXkukTav2^dodr5#9";

    public static class ConfigException extends VCException {
        public ConfigException(String message) {
            super(message);
        }
    }

    public static Either<ConfigException, Void> getConfigAndWriteToFile() {
        Either<ConfigException, Tuple2<ServerConfig, String>> configOrError = getConfigFromServer();
        if (configOrError.isLeft()) {
            return Either.left(configOrError.getLeft());
        }

        Tuple2<ServerConfig, String> config = configOrError.get();

        Either<ConfigException, Void> fileOrError = writeConfigToFile(config._1, config._2);

        if (fileOrError.isLeft()) {
            return Either.left(fileOrError.getLeft());
        }

        return Either.right(null);
    }

    public static Either<ConfigException, Tuple2<ServerConfig, String>> getConfigFromServer() {
        try {
            // Genereate public and private key
            WireguardTunnel.Keypair keypair = WireguardTunnel.generate();
            String publicKey = keypair.getPublicKey();
            String privateKey = keypair.getPrivateKey();

            System.out.println("Public Key: " + publicKey);

            ConfigModels.ClientKey clientKey = new ConfigModels.ClientKey(publicKey, apiKey);

            // Convert to json
            ObjectMapper mapper = new ObjectMapper();
            mapper.configure(Feature.AUTO_CLOSE_SOURCE, true);
            String json = mapper.writeValueAsString(clientKey);
            System.out.println(json);

            HttpClient client = HttpClient.newHttpClient();
            HttpRequest request = HttpRequest.newBuilder().uri(new URI(apiURL))
                    .header("Content-Type", "application/json").version(HttpClient.Version.HTTP_1_1)
                    .POST(BodyPublishers.ofString(json)).build();

            HttpResponse<String> response = client.send(request, HttpResponse.BodyHandlers.ofString());
            // TODO: auto redirect on code 307
            ServerConfig serverConfig = mapper.readValue(response.body(), ServerConfig.class);
            return Either.right(Tuple.of(serverConfig, privateKey));

        } catch (Exception e) {
            System.out.println("Error getting config from server: " + e.getMessage());
            return Either.left(new ConfigException("Failed to get config from server."));
        }
    }

    /*
     * Get default gateway for the system (e.g 192.168.0.1 or 192.168.1.1) important
     * for routing
     */
    static private Either<ConfigException, String> getGateway() {
        try {
            ProcessBuilder builder = new ProcessBuilder("netstat", "-rn");
            Process process = builder.start();

            BufferedReader reader = new BufferedReader(new InputStreamReader(process.getInputStream()));
            String line;

            while ((line = reader.readLine()) != null) {
                if (line.contains("0.0.0.0") || line.startsWith("default")) {
                    String[] parts = line.trim().split("\\s+");
                    if (parts.length >= 3) {
                        System.out.println("Default Gateway: " + parts[2]);
                        return Either.right(parts[2]);
                    }
                }
            }
            System.out.println("No default gateway found.");
            return Either.left(new ConfigException("No default gateway found."));

        } catch (Exception e) {
            System.out.println("Error getting default gateway: " + e.getMessage());
            return Either.left(new ConfigException("No default gateway found."));
        }
    }

    // Writes config file to the same directory as the jar file
    static private Either<ConfigException, Void> writeConfigToFile(ServerConfig serverConfig, String privateKey) {
        Either<ConfigException, String> gatewaySuccessOrFailure = getGateway();

        if (gatewaySuccessOrFailure.isLeft()) {
            return Either.left(gatewaySuccessOrFailure.getLeft());
        }

        String gateway = gatewaySuccessOrFailure.get();

        String jarPath = PathHelpers.getJarPath();

        String remoteIp = serverConfig.getRemoteIp();
        String wstunnelPort = serverConfig.getEndpoint().split(":")[1];
        String wstunnelRemotePort = serverConfig.getWstunnelRemotePort();
        String wstunnelPath = PathHelpers.getWstunnelPath();

        StringBuilder config = new StringBuilder();
        config.append("[Interface]\n");
        config.append("PrivateKey = ").append(privateKey).append("\n");
        config.append("Address = ").append(serverConfig.getAddress()).append("\n");
        config.append("DNS = ").append(serverConfig.getDns()).append("\n");

        // PostUp starts wstunnel and route traffic to it
        config.append("PostUp = ").append(String.format(
                "route add %1$s mask 255.255.255.255 %2$s && start \"\" %5$s client  -L \"udp://%3$s:localhost:%3$s?timeout_sec=0\" wss://%1$s:%4$s",
                remoteIp, gateway, wstunnelPort, wstunnelRemotePort, wstunnelPath)).append("\n");

        // PostDown stops wstunnel and removes the route
        config.append("PostDown = ").append(String.format(
                "route delete %1$s mask 255.255.255.255 %2$s && powershell -command \"(Get-Process -Name wstunnel).Kill()\"",
                remoteIp, gateway)).append("\n\n");

        config.append("[Peer]\n");
        config.append("PublicKey = ").append(serverConfig.getServerKey()).append("\n");
        config.append("Endpoint = ").append(serverConfig.getEndpoint()).append("\n");
        config.append("AllowedIPs = ").append(String.join(",", serverConfig.getAllowedIps())).append("\n");
        config.append("PersistentKeepalive = ").append(serverConfig.getPresKeepAlive()).append("\n");

        PrintWriter out;
        try {
            // Config should be saved in same directory as the jar file
            out = new PrintWriter(PathHelpers.getConfigPath());
            out.println(config);
            out.close();
        } catch (FileNotFoundException e) {
            System.out.println("File not found: " + e.getMessage());
            return Either.left(new ConfigException("Failed to write config."));
        }

        return Either.right(null);

    }

}
