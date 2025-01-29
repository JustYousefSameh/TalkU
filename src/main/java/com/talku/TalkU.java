package com.talku;

import java.awt.Desktop;
import java.awt.SystemTray;
import java.awt.Toolkit;
import java.awt.TrayIcon;
import java.net.URI;
import java.net.URL;
import java.util.Optional;

import org.girod.javafx.svgimage.SVGImage;
import org.girod.javafx.svgimage.SVGLoader;

import com.talku.VC.VCException;

import de.jensd.fx.glyphs.materialicons.MaterialIcon;
import de.jensd.fx.glyphs.materialicons.MaterialIconView;
import io.vavr.control.Either;
import it.sauronsoftware.junique.AlreadyLockedException;
import it.sauronsoftware.junique.JUnique;
import javafx.animation.FadeTransition;
import javafx.animation.Interpolator;
import javafx.animation.ScaleTransition;
import javafx.animation.Transition;
import javafx.animation.TranslateTransition;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.concurrent.Task;
import javafx.concurrent.WorkerStateEvent;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Cursor;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.control.Dialog;
import javafx.scene.control.DialogPane;
import javafx.scene.effect.BlurType;
import javafx.scene.effect.DropShadow;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
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
import javafx.scene.paint.CycleMethod;
import javafx.scene.paint.LinearGradient;
import javafx.scene.paint.Stop;
import javafx.scene.shape.Rectangle;
import javafx.scene.text.Font;
import javafx.scene.text.FontSmoothingType;
import javafx.scene.text.Text;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import javafx.util.Duration;

public class TalkU extends Application {
    private static VC vc;
    private static Thread vcThread;

    public Text oldLabel = new Text("Disconnected");
    public Text newLabel = new Text();

    private TranslateTransition goUpOld = new TranslateTransition(Duration.millis(700), oldLabel);
    private TranslateTransition goUpNew = new TranslateTransition(Duration.millis(700), newLabel);

    private FadeTransition fadeOut = new FadeTransition(Duration.millis(600), oldLabel);
    private FadeTransition fadeIn = new FadeTransition(Duration.millis(600), newLabel);

    private Stage stage;
    private final Rectangle backgroundGradient = new Rectangle();

    private final Color disconnectedColor = Color.valueOf("#350000");
    private final Color connectingColor = Color.valueOf("#1D1A03");
    private final Color connectedColor = Color.valueOf("#082B09");

    private final Color textDisconnectedColor = Color.valueOf("#B81A15");
    private final Color textConnectingColor = Color.valueOf("#B8AA15");
    private final Color textConnectedColor = Color.valueOf("#15B833");

    private Color startColor = disconnectedColor;
    private Color endColor = connectingColor;

    private Stop colorStop = new Stop(1, startColor);

    Task<Void> task;

    private LinearGradient linearGradient = new LinearGradient(0, 0, 1, 1, true, CycleMethod.NO_CYCLE,
            new Stop[] { new Stop(0, Color.BLACK), colorStop });

    private Transition colorTransition = new Transition() {
        {
            setCycleDuration(Duration.millis(2150));
        }

        @Override
        protected void interpolate(double frac) {
            final Color newColor = startColor.interpolate(endColor, frac);
            linearGradient = new LinearGradient(0, 0, 1, 1, true, CycleMethod.NO_CYCLE,
                    new Stop[] { new Stop(0, Color.BLACK), new Stop(1, newColor) });

            backgroundGradient.setFill(linearGradient);
        }
    };

    public static final Interpolator easeInOutBack = new Interpolator() {
        @Override
        protected double curve(double x) {
            final double c1 = 1.70158;
            final double c3 = c1 + 1;

            return 1 + c3 * Math.pow(x - 1, 3) + c1 * Math.pow(x - 1, 2);
        }

        @Override
        public String toString() {
            return "Interpolator.easeInOutBack";
        }
    };

    public void animateLabelChange(String newText) {

        if (goUpNew.getStatus() == javafx.animation.Animation.Status.RUNNING) {
            goUpNew.setOnFinished(event -> {
                animateLabelChange(newText);
            });
            return;
        }

        if (newText.equals("Connected")) {
            newLabel.setFill(textConnectedColor);
        } else if (newText.equals("Disconnected")) {
            newLabel.setFill(textDisconnectedColor);
        } else {
            newLabel.setFill(textConnectingColor);
        }

        newLabel.setTranslateY(50);

        fadeOut.setFromValue(1);
        fadeOut.setToValue(0);
        fadeOut.play();

        goUpOld.setInterpolator(easeInOutBack);
        goUpOld.setToY(-50);
        goUpOld.play();

        goUpOld.setOnFinished(event -> {
            oldLabel.setText(newText);
            oldLabel.setTranslateY(0);
            if (newText.equals("Connected")) {
                oldLabel.setFill(textConnectedColor);
            } else if (newText.equals("Disconnected")) {
                oldLabel.setFill(textDisconnectedColor);
            } else {
                oldLabel.setFill(textConnectingColor);
            }
            oldLabel.setOpacity(1);
            newLabel.setText("");
        });

        newLabel.setText(newText);
        fadeIn.setFromValue(0);
        fadeIn.setToValue(1);
        fadeIn.play();

        goUpNew.setInterpolator(easeInOutBack);
        goUpNew.setToY(0);
        goUpNew.play();
        goUpNew.setOnFinished(null);
    }

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
        if (alreadyRunning) {
            final Dialog<String> dialog = new Dialog<String>();

            final Integer width = 350;
            final Integer height = 150;

            final Color errorColor = Color.valueOf("#D41919");

            Rectangle cropRect = new Rectangle(width, height);
            cropRect.setArcHeight(15.0);
            cropRect.setArcWidth(15.0);

            Button retryButton = new Button("  OK  ");
            retryButton.setPrefHeight(32);
            retryButton.setMaxHeight(32);
            retryButton.setMinHeight(32);

            retryButton.setPadding(new Insets(0, 18, 0, 18));

            retryButton.setStyle(
                    "-fx-background-radius: 10; -fx-background-color: #B81A15; -fx-text-fill: white; -fx-font-size: 17px; -fx-font-weight: 600; -fx-font-family: Calibri;");

            retryButton.setOnAction(event -> {
                dialog.setResult("Retry");
                dialog.close();
            });

            // HBox buttonBox = new HBox(5);
            // buttonBox.getChildren().addAll(retryButton, closeButton);
            // buttonBox.setAlignment(Pos.CENTER_RIGHT);
            // buttonBox.setTranslateY(5);

            Text errorTitle = new Text("Application Error");
            errorTitle.setFontSmoothingType(FontSmoothingType.GRAY);
            errorTitle.setStyle("-fx-font-size: 20px; -fx-font-weight: bold; -fx-font-family: Calibri;");
            errorTitle.setFill(Color.WHITE);

            Text errorContent = new Text("TalkU is already running.");
            errorContent.setFontSmoothingType(FontSmoothingType.GRAY);
            errorContent.setStyle("-fx-font-size: 16px; -fx-font-family: Calibri;");
            errorContent.setFill(Color.WHITE);

            MaterialIconView icon = new MaterialIconView(MaterialIcon.ERROR_OUTLINE);
            icon.setFill(errorColor);
            icon.setGlyphSize(65);

            VBox textVbox = new VBox(6);
            textVbox.getChildren().addAll(errorTitle, errorContent);
            textVbox.setAlignment(Pos.CENTER_LEFT);

            HBox textHbox = new HBox(10);
            textHbox.setAlignment(Pos.CENTER_LEFT);
            textHbox.getChildren().addAll(icon, textVbox);

            VBox dialogBox = new VBox(0);
            dialogBox.setPadding(new Insets(15, 15, 0, 15));
            dialogBox.setAlignment(Pos.CENTER_RIGHT);

            Region spacer = new Region();
            VBox.setVgrow(spacer, Priority.ALWAYS);

            retryButton.setAlignment(Pos.BASELINE_RIGHT);

            dialogBox.getChildren().addAll(textHbox, spacer, retryButton);

            dialog.setTitle("Error");
            dialog.setWidth(width);
            dialog.setHeight(height);

            final DialogPane dialogPane = dialog.getDialogPane();

            dialogPane.setContent(dialogBox);
            dialogPane.getScene().setFill(Color.TRANSPARENT);
            dialogPane.setClip(cropRect);
            // final Background background = new Backgroundj();
            dialogPane.setBackground(
                    new Background(new BackgroundFill(Color.valueOf("#212121"), new CornerRadii(0), new Insets(0))));

            dialogPane.setMaxWidth(width);
            dialogPane.setPrefWidth(width);
            dialogPane.setMinWidth(width);

            dialogPane.setMaxHeight(height);
            dialogPane.setPrefHeight(height);
            dialogPane.setMinHeight(height);

            final Delta dragDelta = new Delta();

            dialogPane.setOnMousePressed(mouseEvenet -> {
                dragDelta.x = dialog.getX() - mouseEvenet.getScreenX();
                dragDelta.y = dialog.getY() - mouseEvenet.getScreenY();

            });

            dialogPane.setOnMouseDragged(mouseEvenet -> {
                dialog.setX(mouseEvenet.getScreenX() + dragDelta.x);
                dialog.setY(mouseEvenet.getScreenY() + dragDelta.y);
            });

            dialog.initStyle(StageStyle.TRANSPARENT);
            dialog.showAndWait();
            Platform.exit();
            return;
        }

        final Task<VC> createVCTask = new Task<VC>() {
            @Override
            protected VC call() throws Exception {
                addAppToTray();
                final VC localVC = new VC();
                return localVC;

            }
        };

        createVCTask.addEventHandler(WorkerStateEvent.WORKER_STATE_SUCCEEDED, event -> {
            vc = createVCTask.getValue();
            vc.labelProperty().addListener((observable, oldValue, newValue) -> {
                animateLabelChange(newValue);
            });
        });

        vcThread = new Thread(createVCTask);
        vcThread.setDaemon(true);
        vcThread.start();

        this.stage = stage;
        stage.setTitle("TalkU");
        stage.setResizable(false);
        stage.initStyle(javafx.stage.StageStyle.TRANSPARENT);
        stage.getIcons().add(new Image("/32.png"));
        Platform.setImplicitExit(false);

        colorTransition.setOnFinished(event -> {
            startColor = linearGradient.getStops().get(1).getColor();
        });

        backgroundGradient.setFill(linearGradient);
        backgroundGradient.setWidth(450);
        backgroundGradient.setHeight(400);

        URL url = getClass().getResource("/world.svg");
        SVGImage svgImage = SVGLoader.load(url);

        Rectangle rect = new Rectangle(450, 400);
        rect.setArcHeight(15.0);
        rect.setArcWidth(15.0);

        Image discordImage = new Image("discord.png");
        ImageView discordImageView = new ImageView(discordImage);
        discordImageView.setFitWidth(32);
        discordImageView.setPreserveRatio(true);

        Button discordButton = new Button("", discordImageView);
        discordButton.setPrefHeight(60);
        discordButton.setPrefWidth(60);
        discordButton.setBackground(Background.EMPTY);
        discordButton.setTranslateX(-5);
        discordButton.setTranslateY(30);
        StackPane.setAlignment(discordButton, Pos.BOTTOM_LEFT);

        Image githubImage = new Image("github.png");
        ImageView githubImageView = new ImageView(githubImage);
        githubImageView.setFitWidth(32);
        githubImageView.setPreserveRatio(true);

        Button githubButton = new Button("", githubImageView);
        githubButton.setPrefHeight(60);
        githubButton.setPrefWidth(60);
        githubButton.setBackground(Background.EMPTY);
        githubButton.setTranslateX(40);
        githubButton.setTranslateY(30);
        StackPane.setAlignment(githubButton, Pos.BOTTOM_LEFT);

        TranslateTransition discordTransition = new TranslateTransition(Duration.millis(300), discordButton);
        discordTransition.setInterpolator(easeInOutBack);

        TranslateTransition githubTransition = new TranslateTransition(Duration.millis(300), githubButton);
        githubTransition.setInterpolator(easeInOutBack);

        githubButton.setOnAction(event -> {
            try {
                Desktop.getDesktop().browse(new URI("https://github.com/JustYousefSameh/TalkU"));
            } catch (Exception e) {

            }
        });

        githubButton.setOnMouseEntered(event -> {
            githubTransition.setToY(10);
            githubTransition.stop();
            githubTransition.play();
            githubButton.setCursor(Cursor.HAND);

        });

        githubButton.setOnMouseExited(event -> {
            githubTransition.setToY(30);
            githubTransition.stop();
            githubTransition.play();
            githubButton.setCursor(Cursor.DEFAULT);
        });

        discordButton.setOnAction(event -> {
            try {
                Desktop.getDesktop().browse(new URI("https://discord.gg/kqY7dM7ekJ"));
            } catch (Exception e) {

            }
        });

        discordButton.setOnMouseEntered(event -> {

            discordTransition.setToY(10);
            discordTransition.stop();
            discordTransition.play();
            discordButton.setCursor(Cursor.HAND);

        });

        discordButton.setOnMouseExited(event -> {
            discordTransition.setToY(30);
            discordTransition.stop();
            discordTransition.play();
            discordButton.setCursor(Cursor.DEFAULT);
        });

        VBox mainVBox = new VBox(80);
        mainVBox.setAlignment(Pos.CENTER);

        TitleBar titleBar = new TitleBar(stage);

        StackPane stackPane = new StackPane();
        stackPane.getChildren().addAll(backgroundGradient, svgImage, mainVBox, discordButton, githubButton, titleBar);
        StackPane.setAlignment(titleBar, Pos.TOP_CENTER);
        stackPane.setClip(rect);

        Scene scene = new Scene(stackPane, 450, 400);
        scene.setFill(Color.TRANSPARENT);

        DropShadow dropShadow = new DropShadow();
        dropShadow.setColor(Color.rgb(0, 0, 0, 0.3));
        dropShadow.setBlurType(BlurType.THREE_PASS_BOX);
        dropShadow.setSpread(0.2);
        dropShadow.setRadius(2.0);
        dropShadow.setOffsetY(5);
        dropShadow.setOffsetX(3);

        Font font = Font.loadFont(getClass().getResourceAsStream("/Product Sans Bold.ttf"), 45);

        oldLabel.setFont(font);
        oldLabel.setFill(textDisconnectedColor);
        oldLabel.setFontSmoothingType(FontSmoothingType.GRAY);
        oldLabel.setEffect(dropShadow);

        newLabel.setFont(font);
        newLabel.setFill(textDisconnectedColor);
        newLabel.setFontSmoothingType(FontSmoothingType.GRAY);
        newLabel.setEffect(dropShadow);

        SwitchButton switchButton = new SwitchButton();

        Rectangle labelClip = new Rectangle(450, 55);

        StackPane labelStackpane = new StackPane();
        labelStackpane.setClip(labelClip);
        labelStackpane.getChildren().addAll(oldLabel, newLabel);

        Rectangle scalingCircle = new Rectangle(180, 80);
        scalingCircle.setArcHeight(80);
        scalingCircle.setArcWidth(80);
        scalingCircle.setStroke(connectedColor);
        scalingCircle.setStrokeWidth(15);
        scalingCircle.setOpacity(0);
        scalingCircle.setFill(Color.TRANSPARENT);

        ScaleTransition circleAnim = new ScaleTransition(Duration.millis(2000), scalingCircle);
        FadeTransition circleFade = new FadeTransition(Duration.millis(2000), scalingCircle);

        StackPane buttonAndAnim = new StackPane();
        buttonAndAnim.getChildren().addAll(scalingCircle, switchButton);

        mainVBox.getChildren().addAll(labelStackpane, buttonAndAnim);

        Runnable onButtonClick = () -> {
            final Boolean state = switchButton.getState();

            new Thread(() -> {
                final Either<VCException, Boolean> valueOrFailure = vc.connect(state);

                if (valueOrFailure.isLeft()) {
                    javafx.application.Platform.runLater(() -> {
                        switchButton.disable();
                        switchButton.setIsClickable(true);

                        animateLabelChange("Disconnected");

                        endColor = disconnectedColor;
                        colorTransition.stop();
                        colorTransition.play();

                        final Dialog<String> dialog = new Dialog<String>();

                        final Integer width = 350;
                        final Integer height = 150;

                        final Color errorColor = Color.valueOf("#D41919");

                        Rectangle cropRect = new Rectangle(width, height);
                        cropRect.setArcHeight(15.0);
                        cropRect.setArcWidth(15.0);

                        Button retryButton = new Button("Retry");
                        retryButton.setPrefHeight(32);
                        retryButton.setMaxHeight(32);
                        retryButton.setMinHeight(32);

                        retryButton.setPadding(new Insets(0, 18, 0, 18));

                        retryButton.setStyle(
                                "-fx-background-radius: 10; -fx-background-color: #B81A15; -fx-text-fill: white; -fx-font-size: 17px; -fx-font-weight: 600; -fx-font-family: Calibri;");

                        retryButton.setOnAction(event -> {
                            dialog.setResult("Retry");
                            dialog.close();
                        });

                        Button closeButton = new Button("Cancel");
                        closeButton.setStyle(
                                "-fx-background-color: transparent; -fx-text-fill: white; -fx-font-size: 17px; -fx-font-weight: 600; -fx-font-family: Calibri;");
                        closeButton.setOnAction(event -> {
                            dialog.setResult("Cancel");
                            dialog.close();
                        });

                        HBox buttonBox = new HBox(5);
                        buttonBox.getChildren().addAll(retryButton, closeButton);
                        buttonBox.setAlignment(Pos.CENTER_RIGHT);
                        buttonBox.setTranslateY(5);

                        Text errorTitle = new Text("Connection Error");
                        errorTitle.setFontSmoothingType(FontSmoothingType.GRAY);
                        errorTitle.setStyle("-fx-font-size: 20px; -fx-font-weight: bold; -fx-font-family: Calibri;");
                        errorTitle.setFill(Color.WHITE);

                        Text errorContent = new Text("Couldn't connect to VPN.");
                        errorContent.setFontSmoothingType(FontSmoothingType.GRAY);
                        errorContent.setStyle("-fx-font-size: 16px; -fx-font-family: Calibri;");
                        errorContent.setFill(Color.WHITE);

                        MaterialIconView icon = new MaterialIconView(MaterialIcon.ERROR_OUTLINE);
                        icon.setFill(errorColor);
                        icon.setGlyphSize(65);

                        VBox textVbox = new VBox(6);
                        textVbox.getChildren().addAll(errorTitle, errorContent);
                        textVbox.setAlignment(Pos.CENTER_LEFT);

                        HBox textHbox = new HBox(10);
                        textHbox.setAlignment(Pos.CENTER_LEFT);
                        textHbox.getChildren().addAll(icon, textVbox);

                        VBox dialogBox = new VBox(0);
                        dialogBox.setPadding(new Insets(15, 15, 0, 15));

                        Region spacer = new Region();
                        VBox.setVgrow(spacer, Priority.ALWAYS);

                        dialogBox.getChildren().addAll(textHbox, spacer, buttonBox);

                        dialog.setTitle("Error");
                        dialog.setWidth(width);
                        dialog.setHeight(height);

                        final DialogPane dialogPane = dialog.getDialogPane();

                        dialogPane.setContent(dialogBox);
                        dialogPane.getScene().setFill(Color.TRANSPARENT);
                        dialogPane.setClip(cropRect);
                        // final Background background = new Backgroundj();
                        dialogPane.setBackground(new Background(
                                new BackgroundFill(Color.valueOf("#212121"), new CornerRadii(0), new Insets(0))));

                        dialogPane.setMaxWidth(width);
                        dialogPane.setPrefWidth(width);
                        dialogPane.setMinWidth(width);

                        dialogPane.setMaxHeight(height);
                        dialogPane.setPrefHeight(height);
                        dialogPane.setMinHeight(height);

                        final Delta dragDelta = new Delta();

                        dialogPane.setOnMousePressed(mouseEvenet -> {
                            dragDelta.x = dialog.getX() - mouseEvenet.getScreenX();
                            dragDelta.y = dialog.getY() - mouseEvenet.getScreenY();

                        });

                        dialogPane.setOnMouseDragged(mouseEvenet -> {
                            dialog.setX(mouseEvenet.getScreenX() + dragDelta.x);
                            dialog.setY(mouseEvenet.getScreenY() + dragDelta.y);
                        });

                        dialog.initStyle(StageStyle.TRANSPARENT);

                        Optional<String> action = dialog.showAndWait();
                        if (action.get() == "Retry") {
                            MouseEvent mouseEvent = new MouseEvent(MouseEvent.MOUSE_CLICKED, 1, 2, 3, 4,
                                    MouseButton.PRIMARY, 1, true, true, true, true, true, true, true, true, true, true,
                                    null);
                            switchButton.fireEvent(mouseEvent);
                        }

                    });

                } else {
                    final Boolean isConnected = valueOrFailure.get();
                    javafx.application.Platform.runLater(() -> {
                        if (isConnected) {
                            animateLabelChange("Connected");
                            endColor = connectedColor;
                            colorTransition.stop();
                            colorTransition.play();
                            switchButton.enable();

                            circleFade.setInterpolator(easeInOutBack);
                            circleFade.setFromValue(0.6);
                            circleFade.setToValue(0);
                            circleAnim.setDelay(Duration.millis(250));
                            circleFade.setDelay(Duration.millis(250));
                            circleAnim.setFromX(1);
                            circleAnim.setFromY(1);
                            circleAnim.setToX(20);
                            circleAnim.setToY(20);
                            circleFade.play();
                            circleAnim.play();

                            // scalingCircle.setVisible(true);
                        } else {
                            // animateLabelChange("Disconnected");
                            // endColor = disconnectedColor;
                            // colorTransition.playFromStart();

                            // switchButton.disable();
                            switchButton.setIsClickable(true);
                        }
                    });
                }
            }).start();

        };
        switchButton.setOnAction(() -> {

            if (switchButton.getState()) {
                endColor = disconnectedColor;
                animateLabelChange("Disconnected");
            } else {
                endColor = connectingColor;
                animateLabelChange("Initializing...");

            }
            colorTransition.stop();
            colorTransition.play();

            if (createVCTask.isRunning()) {
                createVCTask.addEventHandler(WorkerStateEvent.WORKER_STATE_SUCCEEDED, event -> {
                    onButtonClick.run();
                });
                return;
            }
            onButtonClick.run();

        });

        stage.setScene(scene);
        stage.show();
    }

    public static void main(String[] args) {
        System.setProperty("prism.lcdtext", "false");
        launch();
        vcThread.interrupt();
        if (vc != null) {
            vc.close();
        }
    }

}
