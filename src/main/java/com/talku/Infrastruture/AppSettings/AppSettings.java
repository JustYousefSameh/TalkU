package com.talku.Infrastruture.AppSettings;

import java.util.List;

public class AppSettings {
    private boolean autoConnectOnGameLaunch;
    private List<String> gameList;

    public AppSettings() {
    } // Required for JSON deserialization

    public AppSettings(boolean autoConnectOnGameLaunch, List<String> gameList) {
        this.autoConnectOnGameLaunch = autoConnectOnGameLaunch;
        this.gameList = gameList;
    }

    public boolean isAutoConnectOnGameLaunch() {
        return autoConnectOnGameLaunch;
    }

    public void setAutoConnectOnGameLaunch(boolean autoConnectOnGameLaunch) {
        this.autoConnectOnGameLaunch = autoConnectOnGameLaunch;
    }

    public List<String> getGameList() {
        return gameList;
    }

    public void setGameList(List<String> gameList) {
        this.gameList = gameList;
    }
}
