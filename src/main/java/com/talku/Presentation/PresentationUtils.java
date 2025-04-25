package com.talku.Presentation;

import java.util.Optional;

import de.jensd.fx.glyphs.materialicons.MaterialIcon;
import de.jensd.fx.glyphs.materialicons.MaterialIconView;
import javafx.animation.Interpolator;
import javafx.application.Platform;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.control.Button;
import javafx.scene.control.Dialog;
import javafx.scene.control.DialogPane;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Background;
import javafx.scene.layout.BackgroundFill;
import javafx.scene.layout.CornerRadii;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.scene.paint.Color;
import javafx.scene.shape.Rectangle;
import javafx.scene.text.FontSmoothingType;
import javafx.scene.text.Text;
import javafx.stage.StageStyle;

public class PresentationUtils {

    public static void showAppAlreadyRunningDialog() {
        final Dialog<String> dialog = new Dialog<String>();

        final Integer width = 350;
        final Integer height = 150;

        final Color errorColor = Color.valueOf("#D41919");

        Rectangle cropRect = new Rectangle(width, height);
        cropRect.setArcHeight(15.0);
        cropRect.setArcWidth(15.0);

        Button okButton = new Button("  OK  ");
        okButton.setPrefHeight(32);
        okButton.setMaxHeight(32);
        okButton.setMinHeight(32);

        okButton.setPadding(new Insets(0, 18, 0, 18));

        okButton.setStyle(
                "-fx-background-radius: 10; -fx-background-color: #B81A15; -fx-text-fill: white; -fx-font-size: 17px; -fx-font-weight: 600; -fx-font-family: Calibri;");

        okButton.setOnAction(event -> {
            dialog.setResult("Retry");
            dialog.close();
            Platform.runLater(() -> {
                Platform.exit();
            });
        });

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

        okButton.setAlignment(Pos.BASELINE_RIGHT);

        dialogBox.getChildren().addAll(textHbox, spacer, okButton);

        dialog.setTitle("Error");
        dialog.setWidth(width);
        dialog.setHeight(height);

        final DialogPane dialogPane = dialog.getDialogPane();

        dialogPane.setContent(dialogBox);
        dialogPane.getScene().setFill(Color.TRANSPARENT);
        dialogPane.setClip(cropRect);
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
    }

    public static void showErrorDialog(String errorMessage, SwitchButton switchButton) {
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

        Text errorContent = new Text(errorMessage);
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

        Optional<String> action = dialog.showAndWait();
        if (action.get() == "Retry") {
            MouseEvent mouseEvent = new MouseEvent(MouseEvent.MOUSE_CLICKED, 1, 2, 3, 4, MouseButton.PRIMARY, 1, true,
                    true, true, true, true, true, true, true, true, true, null);
            switchButton.fireEvent(mouseEvent);
        }

    }

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
}
