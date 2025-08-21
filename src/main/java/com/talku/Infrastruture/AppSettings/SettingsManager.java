package com.talku.Infrastruture.AppSettings;

import java.io.File;
import java.io.IOException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.talku.Utils.PathHelpers;

public class SettingsManager {
    private static SettingsManager instance;
    private static final ObjectMapper MAPPER = new ObjectMapper();

    private AppSettings config;

    private SettingsManager() {
        loadConfig();
    }

    public static synchronized SettingsManager getInstance() {
        if (instance == null) {
            instance = new SettingsManager();
        }
        return instance;
    }

    public void loadConfig() {
        File file = Path.of(PathHelpers.getAppConfigPath()).toFile();
        if (file.exists()) {
            try {
                config = MAPPER.readValue(file, AppSettings.class);
            } catch (IOException e) {
                e.printStackTrace();
                config = getDefaultConfig();
            }
        } else {
            // config file doesn't exist, get default config and save it
            config = getDefaultConfig();
            saveConfig();
        }
    }

    public void saveConfig() {
        try {
            File file = Path.of(PathHelpers.getAppConfigPath()).toFile();
            file.getParentFile().mkdirs(); // ensure directory exists

            MAPPER.writerWithDefaultPrettyPrinter().writeValue(file, config);
        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    public AppSettings getConfig() {
        return config;
    }

    private AppSettings getDefaultConfig() {
        // Default games when there is no config
        List<String> defaultGames = new ArrayList<>();

        defaultGames.add("valorant");
        defaultGames.add("helldivers2");
        defaultGames.add("overwatch");

        return new AppSettings(false, defaultGames);
    }

    // === Extra API ===

    public void addGame(String game) {
        if (!config.getGameList().contains(game)) {
            config.getGameList().add(game);
            saveConfig();
        }
    }

    public void removeGame(String game) {
        if (config.getGameList().remove(game)) {
            saveConfig();
        }
    }

    public void setAutoConnectOnGameLaunch(boolean enabled) {
        config.setAutoConnectOnGameLaunch(enabled);
        saveConfig();
    }
}
