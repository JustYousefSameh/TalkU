package com.talku.Presentation;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import com.talku.Infrastruture.AppSettings.SettingsManager;
import com.talku.Infrastruture.WinService.ProcessWatcher;

import de.jensd.fx.glyphs.materialicons.MaterialIcon;
import de.jensd.fx.glyphs.materialicons.MaterialIconView;
import javafx.animation.Interpolator;
import javafx.animation.TranslateTransition;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Cursor;
import javafx.scene.control.Button;
import javafx.scene.control.ScrollPane;
import javafx.scene.control.TextField;
import javafx.scene.effect.DropShadow;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;
import javafx.scene.paint.Color;
import javafx.scene.text.FontSmoothingType;
import javafx.scene.text.Text;
import javafx.util.Duration;

public class SettingsMenu extends VBox {
    private final ToggleSwitch startWithWindows;
    private final ToggleSwitch autoConnectOnGameLaunch;

    private final Runnable hideSettings;

    private Duration animationDuration = Duration.millis(250);

    private final ScrollPane gameListScrollable = new ScrollPane();

    private final SettingsManager settingsManager = SettingsManager.getInstance();

    public SettingsMenu(Runnable hideSettings) {
        this.hideSettings = hideSettings;
        getStylesheets().add(getClass().getResource("/css/settings.css").toExternalForm());

        gameListScrollable.getStyleClass().add("games-scroll");
        gameListScrollable.setPadding(new Insets(4, 0, 0, 12));
        gameListScrollable.setFitToWidth(true);

        var config = settingsManager.getConfig();

        startWithWindows = new ToggleSwitch(isStartUpEnabled());
        autoConnectOnGameLaunch = new ToggleSwitch(config.isAutoConnectOnGameLaunch());

        setSpacing(14);
        setPadding(new Insets(16));
        setAlignment(Pos.TOP_LEFT);
        setMaxWidth(320);
        setMaxHeight(320);

        setStyle("""
                    -fx-background-color: rgba(40,40,40,0.8);
                    -fx-background-radius: 16;
                    -fx-border-radius: 16;
                    -fx-border-color: #5a5a5a;
                    -fx-border-width: 1;
                """);
        setEffect(new DropShadow(18, Color.color(0, 0, 0, 0.65)));

        // Title
        Text title = new Text("Settings");
        title.setStyle("-fx-font-size: 15px; -fx-font-weight: bold;");
        title.setFill(Color.WHITE);

        // Exit button
        MaterialIconView exitIcon = new MaterialIconView(MaterialIcon.CLOSE);
        exitIcon.setFontSmoothingType(FontSmoothingType.GRAY);
        exitIcon.setGlyphSize(17);
        exitIcon.setFill(Color.WHITE);

        Button closeBtn = new Button("", exitIcon);
        closeBtn.setStyle("-fx-background-color: transparent;");
        closeBtn.setCursor(Cursor.HAND);
        closeBtn.setOnMouseClicked(e -> closeMenu());

        HBox titleBar = new HBox(title);
        HBox.setHgrow(titleBar, Priority.ALWAYS);
        HBox topBar = new HBox(titleBar, closeBtn);
        topBar.setAlignment(Pos.CENTER_RIGHT);

        // Monitored Games Section
        Text gamesLabel = new Text("Monitored Games");
        gamesLabel.setStyle("-fx-font-size: 13px; -fx-font-weight: bold;");
        gamesLabel.setFill(Color.LIGHTGRAY);

        refreshGameList();

        // === Add Game Input ===
        TextField gameInput = new TextField();
        gameInput.setPromptText("Enter game exe (e.g. Overwatch2.exe)");
        gameInput.setStyle("""
                -fx-background-color: #2a2a2a;
                -fx-text-fill: white;
                -fx-prompt-text-fill: gray;
                -fx-background-radius: 8;
                -fx-border-radius: 8;
                -fx-border-color: #555;
                """);

        Button addBtn = new Button("+");
        addBtn.setStyle("""
                -fx-background-color: #2ea043;
                -fx-text-fill: white;
                -fx-font-weight: bold;
                -fx-background-radius: 8;
                """);
        addBtn.setCursor(Cursor.HAND);

        addBtn.setOnAction(e -> {
            String newGame = gameInput.getText().trim();
            if (!newGame.isEmpty()) {
                settingsManager.addGame(newGame);

                // If auto-connect is enabled, restart the process watcher to account for the
                // new game
                if (isAutoConnectEnabled()) {
                    ProcessWatcher.getInstance().restart();
                }

                gameInput.clear();
            }

            refreshGameList();
        });

        HBox addGameRow = new HBox(8, gameInput, addBtn);
        addGameRow.setAlignment(Pos.CENTER_LEFT);

        getChildren().addAll(topBar, row("Start with Windows", startWithWindows),
                row("Auto connect on game launch", autoConnectOnGameLaunch), gamesLabel, gameListScrollable,
                addGameRow);

        // Toggle listeners
        startWithWindows.selectedProperty().addListener((obs, oldVal, newVal) -> {
            if (newVal) {
                enableStartup();
            } else {
                disableStartup();
            }
        });

        autoConnectOnGameLaunch.selectedProperty().addListener((obs, oldVal, newVal) -> {
            settingsManager.setAutoConnectOnGameLaunch(newVal);

            if (newVal) {
                ProcessWatcher.getInstance().startWatching();
            } else {
                ProcessWatcher.getInstance().stopWatching();
            }
        });

        playIntroIfNeeded();
    }

    private void refreshGameList() {
        VBox gameList = new VBox(6);
        gameList.setPadding(new Insets(4, 0, 4, 0));

        gameListScrollable.setContent(gameList);
        var config = settingsManager.getConfig();
        for (String game : config.getGameList()) {
            HBox gameRow = new HBox(8);

            Text gameText = new Text("• " + game);
            gameText.setFill(Color.WHITE);
            gameText.setStyle("-fx-font-size: 12px;");

            Button removeBtn = new Button("Remove");
            removeBtn.setStyle("""
                        -fx-background-color: #d9534f;
                        -fx-text-fill: white;
                        -fx-font-size: 11px;
                        -fx-padding: 2 8 2 8;
                        -fx-background-radius: 6;
                        -fx-cursor: hand;
                    """);

            removeBtn.setOnMouseEntered(ev -> removeBtn.setStyle("""
                        -fx-background-color: #c9302c;
                        -fx-text-fill: white;
                        -fx-font-size: 11px;
                        -fx-padding: 2 8 2 8;
                        -fx-background-radius: 6;
                        -fx-cursor: hand;
                    """));

            removeBtn.setOnMouseExited(ev -> removeBtn.setStyle("""
                        -fx-background-color: #d9534f;
                        -fx-text-fill: white;
                        -fx-font-size: 11px;
                        -fx-padding: 2 8 2 8;
                        -fx-background-radius: 6;
                        -fx-cursor: hand;
                    """));

            removeBtn.setOnAction(e -> {
                settingsManager.removeGame(game);

                // If auto-connect is enabled, restart the process watcher to account for the
                // deleted game
                if (isAutoConnectEnabled()) {
                    ProcessWatcher.getInstance().restart();
                }

                refreshGameList();
            });

            gameRow.getChildren().addAll(gameText, removeBtn);

            gameList.getChildren().add(gameRow);
        }
    }

    private HBox row(String label, ToggleSwitch toggle) {
        Text lbl = new Text(label);
        lbl.setStyle("-fx-font-size: 13px;");
        lbl.setFill(Color.WHITE);
        HBox box = new HBox(12, lbl, toggle);
        box.setAlignment(Pos.CENTER_LEFT);
        return box;
    }

    private void playIntroIfNeeded() {
        setTranslateX(400);
        TranslateTransition slide = new TranslateTransition(animationDuration, this);
        slide.setToX(0);
        slide.setInterpolator(Interpolator.EASE_OUT);
        slide.play();
    }

    public void closeMenu() {
        TranslateTransition slide = new TranslateTransition(animationDuration, this);
        slide.setToX(400);
        slide.setInterpolator(Interpolator.EASE_IN);

        slide.setOnFinished(e -> hideSettings.run());
        slide.play();
    }

    // ===== STARTUP SHORTCUT MANAGEMENT =====

    private void enableStartup() {
        try {
            Path startupFolder = Path.of(System.getenv("APPDATA"), "Microsoft", "Windows", "Start Menu", "Programs",
                    "Startup");
            Path shortcut = startupFolder.resolve("TalkU.lnk");

            String exePath = Path.of(System.getenv("LOCALAPPDATA"), "TalkU", "TalkU.exe").toAbsolutePath().toString();

            String vbs = """
                    Set oWS = WScript.CreateObject("WScript.Shell")
                    sLinkFile = "%s"
                    Set oLink = oWS.CreateShortcut(sLinkFile)
                    oLink.TargetPath = "%s"
                    oLink.Save
                    """.formatted(shortcut.toString(), exePath);

            File script = File.createTempFile("shortcut", ".vbs");
            Files.writeString(script.toPath(), vbs);

            new ProcessBuilder("wscript", script.getAbsolutePath()).start().waitFor();
            script.delete();

        } catch (IOException | InterruptedException e) {
            e.printStackTrace();
        }
    }

    private boolean isStartUpEnabled() {
        Path startupFolder = Path.of(System.getenv("APPDATA"), "Microsoft", "Windows", "Start Menu", "Programs",
                "Startup");
        Path shortcut = startupFolder.resolve("TalkU.lnk");
        return Files.exists(shortcut);
    }

    private void disableStartup() {
        try {
            Path startupFolder = Path.of(System.getenv("APPDATA"), "Microsoft", "Windows", "Start Menu", "Programs",
                    "Startup");
            Path shortcut = startupFolder.resolve("TalkU.lnk");
            Files.deleteIfExists(shortcut);
        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    // Expose states
    public boolean isStartWithWindowsEnabled() {
        return startWithWindows.isSelected();
    }

    public void setStartWithWindowsEnabled(boolean v) {
        startWithWindows.setSelected(v);
    }

    public boolean isAutoConnectEnabled() {
        return autoConnectOnGameLaunch.isSelected();
    }

    public void setAutoConnectEnabled(boolean v) {
        autoConnectOnGameLaunch.setSelected(v);
    }
}
