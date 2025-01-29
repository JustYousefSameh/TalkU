package com.talku;

import de.jensd.fx.glyphs.materialicons.MaterialIcon;
import de.jensd.fx.glyphs.materialicons.MaterialIconView;
import javafx.event.ActionEvent;
import javafx.geometry.Pos;
import javafx.scene.control.Button;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.paint.Color;
import javafx.scene.text.Font;
import javafx.scene.text.FontSmoothingType;
import javafx.scene.text.Text;
import javafx.stage.Stage;

class Delta {
    double x, y;
}

class TitleBar extends HBox {
    public TitleBar(Stage stage) {
        final int height = 35;
        final int size = 20;

        final Color color = Color.valueOf("#CFCFCF");

        setAlignment(Pos.CENTER);
        MaterialIconView closeImage = new MaterialIconView(MaterialIcon.CLOSE);
        closeImage.setFontSmoothingType(FontSmoothingType.GRAY);
        closeImage.setGlyphSize(size);
        closeImage.setFill(color);

        MaterialIconView minimizeImage = new MaterialIconView(MaterialIcon.REMOVE);
        minimizeImage.setFontSmoothingType(FontSmoothingType.GRAY);
        minimizeImage.setGlyphSize(size);
        minimizeImage.setFill(color);

        Button closeBtn = new Button("", closeImage);
        Button minimizeBtn = new Button("", minimizeImage);

        Text labelName = new Text("TalkU");

        Font talkuFont = Font.loadFont(getClass().getResourceAsStream("/Roboto-Regular.ttf"), 13);
        labelName.setFont(talkuFont);
        labelName.setFill(color);
        labelName.setFontSmoothingType(FontSmoothingType.GRAY);
        labelName.setTranslateX(10);
        labelName.setTranslateY(-2);

        Region paddingRegion = new Region();
        paddingRegion.setPrefWidth(5);

        closeBtn.setStyle("-fx-background-color: transparent;");
        minimizeBtn.setStyle("-fx-background-color: transparent;");

        setPrefHeight(height);
        setMinHeight(height);
        setMaxHeight(height);

        closeBtn.setOnAction((ActionEvent actionEvent) -> {
            stage.hide();
        });

        minimizeBtn.setOnAction((ActionEvent actionEvent) -> {
            stage.setIconified(true);
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

        this.getChildren().addAll(labelName, speratorRegion, minimizeBtn, closeBtn, paddingRegion);
    }
}