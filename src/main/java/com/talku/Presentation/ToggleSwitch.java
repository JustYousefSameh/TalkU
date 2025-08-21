package com.talku.Presentation;

import javafx.animation.Interpolator;
import javafx.animation.TranslateTransition;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.geometry.Insets;
import javafx.scene.Cursor;
import javafx.scene.input.KeyCode;
import javafx.scene.layout.Region;
import javafx.scene.layout.StackPane;
import javafx.scene.paint.Color;
import javafx.scene.shape.Circle;
import javafx.scene.shape.Rectangle;
import javafx.util.Duration;

public class ToggleSwitch extends StackPane {
    private final double W = 40; // smaller width
    private final double H = 22; // smaller height
    private final double KNOB_R = 9; // smaller knob

    private final Rectangle track = new Rectangle(W, H);
    private final Circle knob = new Circle(KNOB_R);

    private final BooleanProperty selected = new SimpleBooleanProperty(false);
    private final TranslateTransition knobAnim = new TranslateTransition(Duration.millis(160), knob);

    public ToggleSwitch(boolean initialState) {
        setPadding(new Insets(2));
        setMinSize(Region.USE_PREF_SIZE, Region.USE_PREF_SIZE);
        setPrefSize(W, H);

        // Track style
        track.setArcWidth(H);
        track.setArcHeight(H);

        // Knob style
        knob.setFill(Color.WHITE);
        knob.setStroke(Color.web("#d0d0d0"));

        getChildren().addAll(track, knob);
        setCursor(Cursor.HAND);
        setFocusTraversable(true);

        // Toggle on click / keyboard
        setOnMouseClicked(e -> setSelected(!isSelected()));
        setOnKeyPressed(e -> {
            if (e.getCode() == KeyCode.SPACE || e.getCode() == KeyCode.ENTER)
                setSelected(!isSelected());
        });

        // Apply initial state immediately (no animation)
        setSelected(initialState);
        updateVisuals(initialState, false);

        // Animate knob + recolor track on state change
        selected.addListener((obs, was, is) -> updateVisuals(is, true));
    }

    private void updateVisuals(boolean isOn, boolean animate) {
        double to = isOn ? (W / 2 - KNOB_R - 2) : -(W / 2 - KNOB_R - 2);

        if (animate) {
            knobAnim.stop();
            knobAnim.setToX(to);
            knobAnim.setInterpolator(Interpolator.EASE_BOTH);
            knobAnim.play();
        } else {
            knob.setTranslateX(to);
        }

        track.setFill(isOn ? Color.web("#2ea043") : Color.web("#3a3a3a"));
        track.setStroke(isOn ? Color.web("#309c47") : Color.web("#5a5a5a"));
    }

    // API
    public boolean isSelected() {
        return selected.get();
    }

    public void setSelected(boolean value) {
        selected.set(value);
    }

    public BooleanProperty selectedProperty() {
        return selected;
    }
}
