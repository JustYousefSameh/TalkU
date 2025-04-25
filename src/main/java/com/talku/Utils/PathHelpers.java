package com.talku.Utils;

import java.nio.file.Paths;

public class PathHelpers {

    public static String getJarPath() {
        String jarPath = null;
        try {
            jarPath = Paths.get(PathHelpers.class.getProtectionDomain().getCodeSource().getLocation().toURI())
                    .toAbsolutePath().toString();
        } catch (Exception e) {
            e.printStackTrace();
        }
        return jarPath;
    }

    public static String getJavaPath() {
        String jarPath = getJarPath();
        String jarDir = jarPath.substring(0, jarPath.lastIndexOf("\\"));
        String talkuPath = jarDir.substring(0, jarDir.lastIndexOf("\\"));
        String javaPath = talkuPath + "\\runtime\\bin\\java.exe";
        return javaPath;
    }

    // Same dir as jar file
    public static String getConfigPath() {
        String jarPath = getJarPath();
        String jarDir = jarPath.substring(0, jarPath.lastIndexOf("\\"));
        String configPath = jarDir + "\\talkuwg.conf";
        return configPath;
    }

    // Same dir as jar file
    public static String getLogPath() {
        String jarPath = getJarPath();
        String jarDir = jarPath.substring(0, jarPath.lastIndexOf("\\"));
        String logPath = jarDir + "\\log.bin"; 
        return logPath;
    }

    // Same dir as jar file
    public static String getWstunnelPath() {
        String jarPath = getJarPath();
        String jarDir = jarPath.substring(0, jarPath.lastIndexOf("\\"));
        String configPath = jarDir + "\\wstunnel.exe";
        return configPath;
    }

    // Same dir as jar file
    public static String getWireguardDllPath() {
        String jarPath = getJarPath();
        String jarDir = jarPath.substring(0, jarPath.lastIndexOf("\\"));
        String wireguardDllpath = jarDir + "\\wireguard.dll";
        return wireguardDllpath;
    }
}
