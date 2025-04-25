package com.talku.Presentation;

import de.jensd.fx.glyphs.fontawesome.FontAwesomeIcon;
import de.jensd.fx.glyphs.fontawesome.FontAwesomeIconView;
import javafx.animation.Animation;
import javafx.animation.Interpolator;
import javafx.animation.RotateTransition;
import javafx.animation.ScaleTransition;
import javafx.animation.Transition;
import javafx.animation.TranslateTransition;
import javafx.event.Event;
import javafx.event.EventHandler;
import javafx.scene.Cursor;
import javafx.scene.effect.BlurType;
import javafx.scene.effect.DropShadow;
import javafx.scene.image.ImageView;
import javafx.scene.layout.StackPane;
import javafx.scene.paint.Color;
import javafx.scene.shape.Circle;
import javafx.scene.shape.Rectangle;
import javafx.util.Duration;

public class SwitchButton extends StackPane {

    private final int width = 180, height = 80, radius = 80, backButtonSpace = 7, glyphSize = 46;
    private final int buttonDiameter = height - backButtonSpace * 2;
    private final int buttonTranslateX = -(width / 2) + buttonDiameter / 2 + backButtonSpace;
    private final Rectangle back = new Rectangle(width, height, Color.RED);

    private FontAwesomeIconView disabledIcon = new FontAwesomeIconView(FontAwesomeIcon.MICROPHONE_SLASH);
    private FontAwesomeIconView enabledIcon = new FontAwesomeIconView(FontAwesomeIcon.MICROPHONE);
    private ImageView loadingIcon = new ImageView("loading.png");

    private final Circle buttonCircle = new Circle(buttonDiameter / 2);
    private final StackPane buttonStack = new StackPane();

    private Runnable function;
    private Boolean isLoading;

    private RotateTransition rotationTransition = new RotateTransition(Duration.millis(1750), loadingIcon);
    private TranslateTransition buttonTranslateTransition = new TranslateTransition(Duration.millis(300), buttonStack);
    private ScaleTransition buttonScaleTransition = new ScaleTransition(Duration.millis(150), this);

    private Transition widthTransition = new Transition() {
        {
            setCycleDuration(Duration.millis(350));
        }

        @Override
        protected void interpolate(double frac) {
            if (isLoading) {
                double newWidth = (height - width) * frac + width;
                back.setWidth(newWidth);
            } else {
                double newWidth = (width - height) * frac + height;
                back.setWidth(newWidth);
            }
        }
    };

    private Color backDisconnectedColor = Color.valueOf("#3B0B0B");
    private Color backConnectingColor = Color.valueOf("#26260A");
    private Color backConnectedColor = Color.valueOf("#0C2810");

    private Color buttonDisconnectedColor = Color.valueOf("#ED4444");
    // private Color buttonConnectingColor = Color.valueOf("#D5DB27");
    private Color buttonConnectedColor = Color.valueOf("#23A446");

    private Color buttonCicleDisconnected = Color.valueOf("#531313");
    private Color buttonCircleConnecting = Color.valueOf("#3C3D11");
    private Color buttonCircleConnected = Color.valueOf("143C1C");

    private boolean isClickable = true;
    private boolean state = false;

    public void setIsClickable(Boolean clickable) {
        this.isClickable = clickable;
    }

    public Boolean getIsClickable() {
        return isClickable;
    }

    public Boolean getState() {
        return state;
    }

    public void setOnAction(Runnable callback) {
        this.function = callback;
    }

    private void init() {
        setMinSize(width, height);
        setMaxSize(width, height);

        rotationTransition.setToAngle(360);
        rotationTransition.setInterpolator(Interpolator.EASE_BOTH);
        rotationTransition.setCycleCount(Animation.INDEFINITE);

        disabledIcon.setGlyphSize(glyphSize);
        disabledIcon.setFill(buttonDisconnectedColor);

        loadingIcon.setFitHeight(52);
        loadingIcon.setPreserveRatio(true);

        enabledIcon.setGlyphSize(glyphSize);
        enabledIcon.setFill(buttonConnectedColor);

        back.maxWidth(width);
        back.minWidth(width);
        back.maxHeight(height);
        back.minHeight(height);
        back.setArcHeight(radius);
        back.setArcWidth(radius);
        back.setFill(backDisconnectedColor);

        buttonStack.setTranslateX(buttonTranslateX);
        buttonStack.setMaxWidth(buttonDiameter);
        buttonStack.setPrefWidth(buttonDiameter);
        buttonStack.setMinWidth(buttonDiameter);
        buttonCircle.setFill(buttonCicleDisconnected);

        DropShadow normalDropShadow = new DropShadow();
        normalDropShadow.setColor(Color.rgb(0, 0, 0, 0.25));
        normalDropShadow.setBlurType(BlurType.THREE_PASS_BOX);
        normalDropShadow.setSpread(0.2);
        normalDropShadow.setRadius(2);
        normalDropShadow.setOffsetY(5);
        normalDropShadow.setOffsetX(3);
        back.setEffect(normalDropShadow);

        buttonStack.getChildren().addAll(buttonCircle, disabledIcon);
        getChildren().addAll(back, buttonStack);

    }

    public void enable() {
        isLoading = false;
        widthTransition.play();
        rotationTransition.stop();
        loadingIcon.setRotate(0);

        ScaleTransition scaleTransition = new ScaleTransition(Duration.millis(250), this);
        scaleTransition.setDelay(Duration.millis(150));
        scaleTransition.setToX(1.03);
        scaleTransition.setToY(1.03);
        scaleTransition.setCycleCount(2);
        scaleTransition.setAutoReverse(true);
        scaleTransition.play();

        back.setFill(backConnectedColor);
        buttonCircle.setFill(buttonCircleConnected);
        buttonStack.getChildren().set(1, enabledIcon);

        buttonTranslateTransition.stop();
        buttonTranslateTransition.setToX(-buttonTranslateX);
        buttonTranslateTransition.play();

        state = true;
        isClickable = true;
    }

    public void disable() {
        isLoading = false;
        back.setFill(backDisconnectedColor);
        buttonCircle.setFill(buttonCicleDisconnected);
        buttonStack.getChildren().set(1, disabledIcon);

        if (back.getWidth() != width) {
            widthTransition.stop();
            widthTransition.play();
        }

        buttonTranslateTransition.stop();
        buttonTranslateTransition.setToX(buttonTranslateX);
        buttonTranslateTransition.play();

        state = false;

    }

    public SwitchButton() {
        init();
        EventHandler<Event> click = (Event e) -> {
            if (!isClickable) {
                return;
            }
            isClickable = false;
            buttonScaleTransition.setToX(1.0);
            buttonScaleTransition.setToY(1.0);
            buttonScaleTransition.stop();
            buttonScaleTransition.play();
            setCursor(Cursor.DEFAULT);

            function.run();

            if (state) {
                disable();
            } else {
                isLoading = true;
                back.setFill(backConnectingColor);
                buttonCircle.setFill(buttonCircleConnecting);
                buttonStack.getChildren().set(1, loadingIcon);
                buttonTranslateTransition.setToX(0);
                buttonTranslateTransition.play();

                rotationTransition.play();
                widthTransition.play();
            }
        };

        setOnMouseEntered(e -> {
            if (!isClickable) {
                return;
            }
            buttonScaleTransition.setToX(1.07);
            buttonScaleTransition.setToY(1.07);
            buttonScaleTransition.play();
            setCursor(Cursor.HAND);
        });

        setOnMouseExited(e -> {
            if (buttonScaleTransition.getStatus() == Animation.Status.RUNNING) {
                buttonScaleTransition.stop();
            }
            buttonScaleTransition.setToX(1.0);
            buttonScaleTransition.setToY(1.0);
            buttonScaleTransition.play();
            setCursor(Cursor.DEFAULT);

        });

        setOnMouseClicked(click);
    }
}
