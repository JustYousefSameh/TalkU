package com.talku.Presentation;

import de.jensd.fx.glyphs.materialicons.MaterialIcon;
import de.jensd.fx.glyphs.materialicons.MaterialIconView;
import javafx.event.ActionEvent;
import javafx.geometry.Pos;
import javafx.scene.Cursor;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.paint.Color;
import javafx.scene.shape.Rectangle;
import javafx.scene.text.Font;
import javafx.scene.text.FontSmoothingType;
import javafx.scene.text.Text;
import javafx.stage.Stage;

class Delta {
    double x, y;
}

class TitleBar extends HBox {
    public TitleBar(Stage stage, Runnable toggleSettings) {
        final int height = 35;
        final int size = 20;

        setSpacing(4);
        setPadding(new javafx.geometry.Insets(0, 8, 0, 8));

        final Color color = Color.valueOf("#CFCFCF");

        setAlignment(Pos.CENTER);
        MaterialIconView closeImage = new MaterialIconView(MaterialIcon.CLOSE);
        closeImage.setFontSmoothingType(FontSmoothingType.GRAY);
        closeImage.setGlyphSize(size);
        closeImage.setFill(color);

        Button closeBtn = new Button("", closeImage);

        MaterialIconView minimizeImage = new MaterialIconView(MaterialIcon.REMOVE);
        minimizeImage.setFontSmoothingType(FontSmoothingType.GRAY);
        minimizeImage.setGlyphSize(size);
        minimizeImage.setFill(color);

        Button minimizeBtn = new Button("", minimizeImage);

        MaterialIconView settingsImage = new MaterialIconView(MaterialIcon.SETTINGS);
        settingsImage.setFontSmoothingType(FontSmoothingType.GRAY);
        settingsImage.setGlyphSize(size);
        settingsImage.setFill(color);

        Button settingsBtn = new Button("", settingsImage);

        Text labelName = new Text("TalkU");

        VersionViewer versionViewer = new VersionViewer("v2.3");
        versionViewer.setTranslateY(-1);

        Font talkuFont = Font.loadFont(getClass().getResourceAsStream("/Roboto-Regular.ttf"), 13);
        labelName.setFont(talkuFont);
        labelName.setFill(color);
        labelName.setFontSmoothingType(FontSmoothingType.GRAY);
        labelName.setTranslateY(-2);

        closeBtn.setStyle("-fx-background-color: transparent;");
        minimizeBtn.setStyle("-fx-background-color: transparent;");
        settingsBtn.setStyle("-fx-background-color: transparent;");

        makeButtonMouseInteractive(closeBtn);
        makeButtonMouseInteractive(minimizeBtn);
        makeButtonMouseInteractive(settingsBtn);

        setPrefHeight(height);
        setMinHeight(height);
        setMaxHeight(height);

        closeBtn.setOnAction((ActionEvent actionEvent) -> {
            stage.hide();
        });

        minimizeBtn.setOnAction((ActionEvent actionEvent) -> {
            stage.setIconified(true);
        });

        settingsBtn.setOnAction((ActionEvent actionEvent) -> {
            toggleSettings.run();
        });

        Region speratorRegion = new Region();
        setHgrow(speratorRegion, Priority.ALWAYS);

        final Delta dragDelta = new Delta();

        setOnMousePressed(mouseEvenet -> {
            dragDelta.x = stage.getX() - mouseEvenet.getScreenX();
            dragDelta.y = stage.getY() - mouseEvenet.getScreenY();

        });

        setOnMouseDragged(mouseEvenet -> {
            stage.setX(mouseEvenet.getScreenX() + dragDelta.x);
            stage.setY(mouseEvenet.getScreenY() + dragDelta.y);
        });

        this.getChildren().addAll(labelName, versionViewer, speratorRegion, settingsBtn, minimizeBtn, closeBtn);
    }

    private void makeButtonMouseInteractive(Node button) {
        button.setOnMouseEntered(e -> {
            button.setCursor(Cursor.HAND);
        });

        button.setOnMouseExited(e -> {
            button.setCursor(Cursor.DEFAULT);
        });
    }
}
