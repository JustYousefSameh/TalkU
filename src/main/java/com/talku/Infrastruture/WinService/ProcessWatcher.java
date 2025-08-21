package com.talku.Infrastruture.WinService;

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.util.List;

import com.talku.Infrastruture.AppSettings.SettingsManager;
import com.talku.Presentation.SwitchButton;

import javafx.application.Platform;

public class ProcessWatcher {

    private static ProcessWatcher instance;

    // This is used to init the singleton
    // Should only be called once
    public static ProcessWatcher init(SwitchButton switchButton) {
        instance = new ProcessWatcher(switchButton);
        return instance;
    }

    public static ProcessWatcher getInstance() {
        if (instance == null) {
            throw new IllegalStateException("ProcessWatcher is not initialized");
        }
        return instance;
    }

    private Thread watcherThread;

    private int sleepDuration = 10000;

    private SwitchButton switchButton;

    // List of the games to be watched
    private List<String> processesToWatch = SettingsManager.getInstance().getConfig().getGameList();

    // If a game is run and caught through ProcessWatcher, this will be set to the
    // name of the game
    // This game will be watched until closed, which will trigger the
    // toggleConnection action (Disconnected the VPN)
    private String gameToWatch;

    public ProcessWatcher(SwitchButton switchButton) {
        this.switchButton = switchButton;
    }

    public void startWatching() {
        // List of the games to be watched
        processesToWatch = SettingsManager.getInstance().getConfig().getGameList();

        // Convert all process names to lower case
        processesToWatch = processesToWatch.stream().map(String::toLowerCase).toList();

        watcherThread = new Thread(() -> {
            try {
                while (!Thread.currentThread().isInterrupted()) {

                    // watch the game caught by the process watcher
                    // on game closed, disconnect the VPN
                    if (gameToWatch != null) {
                        if (isGameClosed()) {
                            Platform.runLater(() -> {

                                System.out.println("Disconnecting VPN...");
                                // If VPN is connected and the game has been closed, disconnect
                                if (switchButton.getState() == true) {
                                    switchButton.simulateClick();
                                }
                            });
                        }

                        Thread.sleep(sleepDuration);
                        continue;
                    }

                    // if VPN is connected (User manually connected) don't listen to games.
                    if (switchButton.getState() == true) {
                        Thread.sleep(sleepDuration);
                        continue;
                    }

                    if (isAnyGameRunning()) {
                        Platform.runLater(() -> {
                            // If VPN is not connected and a watched game is ran, connect
                            if (switchButton.getState() == false) {
                                switchButton.simulateClick();
                            }
                        });

                        Thread.sleep(sleepDuration);
                        continue;
                    }

                    Thread.sleep(sleepDuration);
                }

            } catch (InterruptedException ie) {
                System.out.println("Process watcher stopped due to interruption");
            } catch (Exception e) {
                System.out.println("Error occurred while watching process: " + e.getMessage());
            }
        });

        watcherThread.setDaemon(true);
        watcherThread.start();
    }

    public void stopWatching() {
        if (watcherThread != null) {
            watcherThread.interrupt();
            watcherThread = null;
        }

        gameToWatch = null;
    }

    public void restart() {
        try {
            stopWatching();
            Thread.sleep(100);
            startWatching();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    private boolean isGameClosed() throws Exception {
        System.out.println("Checking if game is closed");
        ProcessBuilder builder = new ProcessBuilder("tasklist");
        Process process = builder.start();

        try {
            BufferedReader reader = new BufferedReader(new InputStreamReader(process.getInputStream()));
            String line;

            while ((line = reader.readLine()) != null) {
                line = line.toLowerCase();
                if (line.contains(gameToWatch)) {
                    return false;
                }
            }
            reader.close();
        } finally {

        }

        System.out.println(gameToWatch + " is closed");
        gameToWatch = null;

        return true;
    }

    private boolean isAnyGameRunning() throws Exception {
        System.out.println("Checking processs...");
        ProcessBuilder builder = new ProcessBuilder("tasklist");
        Process process = builder.start();
        try (BufferedReader reader = new BufferedReader(new InputStreamReader(process.getInputStream()))) {
            String line;
            while ((line = reader.readLine()) != null) {
                line = line.toLowerCase();

                for (String processToWatch : processesToWatch) {
                    if (line.contains(processToWatch)) {
                        System.out.println(processToWatch + " is running");
                        gameToWatch = processToWatch;
                        return true;
                    }
                }
            }
            reader.close();
        }
        return false;
    }
}
