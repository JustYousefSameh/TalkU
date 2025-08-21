package com.talku.Presentation;

import java.awt.SystemTray;
import java.awt.Toolkit;
import java.awt.TrayIcon;
import java.net.URL;

import com.talku.Controller.TalkUController;
import com.talku.Controller.TalkUController.VCException;
import com.talku.Infrastruture.AppSettings.SettingsManager;
import com.talku.Infrastruture.WinService.ProcessWatcher;
import com.talku.Infrastruture.Wireguard.WireGuardTunnelService;

import io.vavr.control.Either;
import javafx.animation.ScaleTransition;
import javafx.animation.TranslateTransition;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.image.Image;
import javafx.scene.layout.BorderPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.StackPane;
import javafx.scene.layout.VBox;
import javafx.scene.paint.Color;
import javafx.scene.shape.Rectangle;
import javafx.stage.Stage;
import javafx.util.Duration;

public class TalkU extends Application {
    static private Stage stage;

    private ActiveCount activeCount;
    private SettingsMenu settingsMenu;

    static public void showStage() {
        System.out.println("Showing stage");
        stage.show();

        // A hack to ensure the stage is on top
        if (!stage.isIconified()) {
            stage.setIconified(true);
        }

        stage.setIconified(false);
        stage.toFront();
    }

    private void addAppToTray() {
        try {
            if (!SystemTray.isSupported()) {
                System.out.println("SystemTray is not supported");
                return;
            }

            URL url = getClass().getResource("/16.png");
            java.awt.Image image = Toolkit.getDefaultToolkit().getImage(url);
            final TrayIcon trayIcon = new TrayIcon(image, "TalkU");
            final SystemTray tray = SystemTray.getSystemTray();

            trayIcon.addActionListener(event -> Platform.runLater(() -> {
                showStage();
            }));

            java.awt.MenuItem openItem = new java.awt.MenuItem("Show");
            openItem.addActionListener(event -> Platform.runLater(() -> {
                showStage();
            }));

            java.awt.MenuItem exitItem = new java.awt.MenuItem("Exit");
            exitItem.addActionListener(event -> {
                Platform.exit();
                tray.remove(trayIcon);
            });

            final java.awt.PopupMenu popup = new java.awt.PopupMenu();
            popup.add(openItem);
            popup.add(exitItem);
            trayIcon.setPopupMenu(popup);

            tray.add(trayIcon);

        } catch (Exception e) {
            System.out.println("TrayIcon could not be added.");
        }

    }

    @Override
    public void start(Stage stage) {

        Platform.setImplicitExit(false);

        TalkU.stage = stage;
        stage.setTitle("TalkU");
        stage.setResizable(false);
        stage.initStyle(javafx.stage.StageStyle.TRANSPARENT);
        stage.getIcons().add(new Image("/32.png"));

        Rectangle rect = new Rectangle(450, 400);
        rect.setArcHeight(15.0);
        rect.setArcWidth(15.0);

        HBox bottomBar = new HBox();
        bottomBar.setMaxHeight(10);
        bottomBar.setAlignment(Pos.BOTTOM_CENTER);

        Region spacer = new Region();
        HBox.setHgrow(spacer, Priority.ALWAYS);

        StackPane.setAlignment(bottomBar, Pos.BOTTOM_CENTER);

        SocialButton githubButton = SocialButton.githubButton();
        SocialButton discordButton = SocialButton.discordButton();

        BorderPane mainVBox = new BorderPane();

        activeCount = new ActiveCount();

        TranslateTransition openingSlideUp = new TranslateTransition();
        openingSlideUp.setDuration(Duration.millis(600));
        openingSlideUp.setNode(bottomBar);
        openingSlideUp.setFromY(30);
        openingSlideUp.setToY(-5);
        openingSlideUp.setInterpolator(PresentationUtils.easeInOutBack);

        bottomBar.getChildren().addAll(discordButton, githubButton, spacer, activeCount);

        StackPane applicationStackPane = new StackPane();

        TitleBar titleBar = new TitleBar(stage, () -> {

            if (settingsMenu == null) {
                settingsMenu = new SettingsMenu(
                        // pass runnable to remove the settings menu from the tree on close
                        () -> {
                            if (settingsMenu != null) {
                                applicationStackPane.getChildren().remove(settingsMenu);
                                settingsMenu = null;
                            }
                        });
                applicationStackPane.getChildren().add(settingsMenu);
            } else {
                // Call close menu on existing settings menu to trigger the animation
                settingsMenu.closeMenu();
            }

        });
        StackPane.setAlignment(titleBar, Pos.TOP_CENTER);

        TranslateTransition slideDownTitleBar = new TranslateTransition();
        slideDownTitleBar.setNode(titleBar);
        slideDownTitleBar.setDuration(Duration.millis(600));
        slideDownTitleBar.setFromY(-30);
        slideDownTitleBar.setToY(0);
        slideDownTitleBar.setInterpolator(PresentationUtils.easeInOutBack);

        GradientBackgroundWithImage gradientBackgroundWithImage = new GradientBackgroundWithImage();

        applicationStackPane.getChildren().addAll(gradientBackgroundWithImage, mainVBox);
        applicationStackPane.setClip(rect);

        Scene scene = new Scene(applicationStackPane, 450, 400);
        scene.setFill(Color.TRANSPARENT);

        AnimatedText animatedText = new AnimatedText();

        SwitchButton switchButton = new SwitchButton();
        ScaleTransition openingScaleTransition = new ScaleTransition();
        openingScaleTransition.setNode(switchButton);
        openingScaleTransition.setDuration(Duration.millis(600));
        openingScaleTransition.setFromX(0);
        openingScaleTransition.setFromY(0);
        openingScaleTransition.setToX(1);
        openingScaleTransition.setToY(1);
        openingScaleTransition.setInterpolator(PresentationUtils.easeInOutBack);

        ScalingCircle scalingCircle = new ScalingCircle();
        StackPane buttonAndCircle = new StackPane();

        buttonAndCircle.getChildren().addAll(scalingCircle, switchButton);

        VBox statusWithButtom = new VBox(animatedText, buttonAndCircle);
        statusWithButtom.setAlignment(Pos.CENTER);
        statusWithButtom.setSpacing(80);

        mainVBox.setCenter(statusWithButtom);
        mainVBox.setTop(titleBar);
        mainVBox.setBottom(bottomBar);

        Runnable onButtonClick = () -> {

            final Boolean state = switchButton.getState();

            if (state) {
                gradientBackgroundWithImage.disconnected();
                animatedText.setText("Disconnected");
            } else {
                gradientBackgroundWithImage.connecting();
                animatedText.setText("Connecting...");
            }

            new Thread(() -> {
                final Either<VCException, Boolean> valueOrFailure = TalkUController.connect(state);

                if (valueOrFailure.isLeft()) {
                    javafx.application.Platform.runLater(() -> {
                        switchButton.disable();
                        switchButton.setIsClickable(true);

                        animatedText.setText("Disconnected");
                        gradientBackgroundWithImage.disconnected();

                        PresentationUtils.showErrorDialog(valueOrFailure.getLeft().getMessage(), switchButton);
                    });
                } else {
                    final Boolean isConnected = valueOrFailure.get();
                    javafx.application.Platform.runLater(() -> {
                        if (isConnected) {
                            switchButton.enable();
                            activeCount.setUserEffectiveCount(1);

                            animatedText.setText("Connected");
                            gradientBackgroundWithImage.connected();
                            scalingCircle.animate();
                        } else {
                            activeCount.setUserEffectiveCount(0);

                            // No need to call diasble as it happens automatically from the switch button
                            // Because disconnecting is instant no need to wait for confirmation
                            switchButton.setIsClickable(true);
                        }
                    });
                }
            }).start();
        };
        switchButton.setOnAction(onButtonClick);

        showStage();
        stage.setScene(scene);

        javax.swing.SwingUtilities.invokeLater(this::addAppToTray);

        openingScaleTransition.play();
        openingSlideUp.play();
        slideDownTitleBar.play();

        animatedText.setText("Disconnected");

        var processWatcher = ProcessWatcher.init(switchButton);

        if (SettingsManager.getInstance().getConfig().isAutoConnectOnGameLaunch()) {
            processWatcher.startWatching();
        }

    }

    @Override
    public void stop() throws Exception {
        activeCount.close();
        ProcessWatcher.getInstance().stopWatching();
        super.stop();
    }

    public static void main(String[] args) {
        System.setProperty("prism.lcdtext", "false");
        launch();

        // when app is closed make sure the service is stopepd and deleted
        WireGuardTunnelService service = new WireGuardTunnelService();
        service.stop();
        service.uninstall();
    }

}
