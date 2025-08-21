package com.talku.Infrastruture.WinService;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.PrintWriter;
import java.net.InetAddress;
import java.net.ServerSocket;
import java.net.Socket;

public class InstanceManager {

    private static InstanceManager instance;

    private InstanceManager() {
    }

    public static InstanceManager getInstance() {
        if (instance == null) {
            instance = new InstanceManager();
        }
        return instance;
    }

    private static final int PORT = 23789; // pick any unused port

    public boolean isPrimaryInstance(Runnable onMessage) {
        try {
            ServerSocket server = new ServerSocket(PORT, 50, InetAddress.getByName("127.0.0.1"));
            System.out.println("This is the first instance. Listening for commands...");

            // Run socket listener on another thread
            Thread listener = new Thread(() -> {
                while (true) {
                    try {
                        Socket client = server.accept();
                        BufferedReader in = new BufferedReader(new InputStreamReader(client.getInputStream()));
                        String line = in.readLine();
                        System.out.println("Received message: " + line);
                        if ("SHOW".equalsIgnoreCase(line)) {
                            onMessage.run(); // run handler (e.g., show window)
                        }
                        in.close();
                    } catch (IOException e) {
                        System.err.println("Error while handling socket: " + e.getMessage());
                    }
                }
            });
            listener.setDaemon(true);
            listener.start();
            return true;

        } catch (IOException e) {
            // Port already in use → another instance is running
            return false;
        }
    }

    public void sendShowMessage() {
        try (Socket socket = new Socket("127.0.0.1", PORT);
                PrintWriter out = new PrintWriter(socket.getOutputStream(), true)) {
            out.println("SHOW");
        } catch (IOException e) {
            System.err.println("Could not connect to primary instance: " + e.getMessage());
        }
    }
}
