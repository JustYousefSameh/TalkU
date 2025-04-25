package com.talku.Infrastruture.VpnConfig;

import java.util.List;

public class ConfigModels {
    static public class ClientKey {
        private String apiKey;
        private String clientPubKey;

        public ClientKey() {
        }

        public ClientKey(String clientPubKey, String apiKey) {
            this.apiKey = apiKey;
            this.clientPubKey = clientPubKey;
        }

        public String getClientPubKey() {
            return clientPubKey;
        }

        public void setClientPubKey(String clientPubKey) {
            this.clientPubKey = clientPubKey;
        }

        public String getApiKey() {
            return apiKey;
        }

        public void setApiKey(String apiKey) {
            this.apiKey = apiKey;
        }
    }

    static public class ServerConfig {
        private String serverKey;
        private String address;
        private List<String> allowedIps;
        private String remoteIp;
        private String endpoint;
        private String presKeepAlive;
        private String dns;
        private String wstunnelRemotePort;

        ServerConfig() {
        }

        public ServerConfig(String serverKey, String address, List<String> allowedIps, String remoteIp, String endpoint,
                String presKeepAlive, String dns, String wstunnelRemotePort) {
            this.serverKey = serverKey;
            this.address = address;
            this.allowedIps = allowedIps;
            this.remoteIp = remoteIp;
            this.endpoint = endpoint;
            this.presKeepAlive = presKeepAlive;
            this.dns = dns;
            this.wstunnelRemotePort = wstunnelRemotePort;
        }

        public String getServerKey() {
            return serverKey;
        }

        public void setServerKey(String serverKey) {
            this.serverKey = serverKey;
        }

        public String getAddress() {
            return address;
        }

        public void setAddress(String address) {
            this.address = address;
        }

        public List<String> getAllowedIps() {
            return allowedIps;
        }

        public void setAllowedIps(List<String> allowedIps) {
            this.allowedIps = allowedIps;
        }

        public String getRemoteIp() {
            return remoteIp;
        }

        public void setRemoteIp(String remoteIp) {
            this.remoteIp = remoteIp;
        }

        public String getEndpoint() {
            return endpoint;
        }

        public void setEndpoint(String endpoint) {
            this.endpoint = endpoint;
        }

        public String getPresKeepAlive() {
            return presKeepAlive;
        }

        public void setPresKeepAlive(String presKeepAlive) {
            this.presKeepAlive = presKeepAlive;
        }

        public String getDns() {
            return dns;
        }

        public void setDns(String dns) {
            this.dns = dns;
        }

        public String getWstunnelRemotePort() {
            return wstunnelRemotePort;
        }

        public void setWstunnelRemotePort(String wstunnelRemotePort) {
            this.wstunnelRemotePort = wstunnelRemotePort;
        }
    }
}
