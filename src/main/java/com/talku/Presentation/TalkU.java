package com.talku.Presentation;

import java.awt.SystemTray;
import java.awt.Toolkit;
import java.awt.TrayIcon;
import java.net.URL;
import java.util.Optional;

import com.talku.Controller.TalkUController;
import com.talku.Controller.TalkUController.VCException;
import com.talku.Infrastruture.Wireguard.WireGuardTunnelService;

import de.jensd.fx.glyphs.materialicons.MaterialIcon;
import de.jensd.fx.glyphs.materialicons.MaterialIconView;
import io.vavr.control.Either;
import it.sauronsoftware.junique.AlreadyLockedException;
import it.sauronsoftware.junique.JUnique;
import javafx.animation.FadeTransition;
import javafx.animation.ScaleTransition;
import javafx.animation.TranslateTransition;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.control.Dialog;
import javafx.scene.control.DialogPane;
import javafx.scene.image.Image;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Background;
import javafx.scene.layout.BackgroundFill;
import javafx.scene.layout.CornerRadii;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.StackPane;
import javafx.scene.layout.VBox;
import javafx.scene.paint.Color;
import javafx.scene.shape.Rectangle;
import javafx.scene.text.FontSmoothingType;
import javafx.scene.text.Text;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import javafx.util.Duration;

public class TalkU extends Application {
    private Stage stage;

    private void showStage() {
        stage.show();
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

            trayIcon.addActionListener(event -> Platform.runLater(this::showStage));

            java.awt.MenuItem openItem = new java.awt.MenuItem("Show");
            openItem.addActionListener(event -> Platform.runLater(this::showStage));

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

        String appId = "TalkU";

        boolean alreadyRunning;
        try {
            JUnique.acquireLock(appId);
            alreadyRunning = false;
        } catch (AlreadyLockedException e) {
            alreadyRunning = true;
        }

        // Show a dialog to inform the user that the app is already running
        if (alreadyRunning) {
            // Not the best way as it loads javafx just to show the error, but I want the
            // error to look good so I'm leaving it
            PresentationUtils.showAppAlreadyRunningDialog();
        }

        if (!alreadyRunning)
            addAppToTray();

        this.stage = stage;
        stage.setTitle("TalkU");
        stage.setResizable(false);
        stage.initStyle(javafx.stage.StageStyle.TRANSPARENT);
        stage.getIcons().add(new Image("/32.png"));

        // when set to false Prevents the app from closing when the last window is
        // closed (Important for tray)
        // If app is alraedy running then we need to be able to close the app
        Platform.setImplicitExit(alreadyRunning);

        Rectangle rect = new Rectangle(450, 400);
        rect.setArcHeight(15.0);
        rect.setArcWidth(15.0);

        SocialButton githubButton = SocialButton.githubButton();
        SocialButton discordButton = SocialButton.discordButton();

        VBox mainVBox = new VBox(80);
        mainVBox.setAlignment(Pos.CENTER);

        TitleBar titleBar = new TitleBar(stage);
        GradientBackgroundWithImage gradientBackgroundWithImage = new GradientBackgroundWithImage();

        StackPane applicationStackPane = new StackPane();
        StackPane.setAlignment(titleBar, Pos.TOP_CENTER);
        applicationStackPane.getChildren().addAll(gradientBackgroundWithImage, mainVBox, discordButton, githubButton,
                titleBar);
        applicationStackPane.setClip(rect);

        Scene scene = new Scene(applicationStackPane, 450, 400);
        scene.setFill(Color.TRANSPARENT);

        AnimatedText animatedText = new AnimatedText();
        SwitchButton switchButton = new SwitchButton();

        ScalingCircle scalingCircle = new ScalingCircle();
        StackPane buttonAndCircle = new StackPane();

        buttonAndCircle.getChildren().addAll(scalingCircle, switchButton);

        mainVBox.getChildren().addAll(animatedText, buttonAndCircle);

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

                            animatedText.setText("Connected");
                            gradientBackgroundWithImage.connected();
                            scalingCircle.animate();
                        } else {
                            // No need to call diasble as it happens automatically from the switch button
                            // Because disconnecting is instant no need to wait for confirmation
                            switchButton.setIsClickable(true);
                        }
                    });
                }
            }).start();
        };
        switchButton.setOnAction(onButtonClick);

        stage.setScene(scene);
        stage.show();
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
