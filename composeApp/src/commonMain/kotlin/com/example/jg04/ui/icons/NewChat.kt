package com.example.jg04.ui.icons

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathFillType
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.path
import androidx.compose.ui.unit.dp

@Suppress("CheckReturnValue")
public val newChatIcon: ImageVector
    get() {
        if (_newChatIcon != null) {
            return _newChatIcon!!
        }
        _newChatIcon =
            ImageVector.Builder(
                name = "chat_add_on",
                defaultWidth = 24.dp,
                defaultHeight = 24.dp,
                viewportWidth = 24f,
                viewportHeight = 24f,
            )
                .apply {
                    path(
                        fill = SolidColor(Color.Black),
                        fillAlpha = 1f,
                        stroke = null,
                        strokeAlpha = 1f,
                        strokeLineWidth = 1f,
                        strokeLineCap = StrokeCap.Butt,
                        strokeLineJoin = StrokeJoin.Bevel,
                        strokeLineMiter = 1f,
                        pathFillType = PathFillType.Companion.NonZero,
                    ) {
                        moveTo(3f, 20f)
                        verticalLineTo(5f)
                        quadTo(3f, 4.17f, 3.59f, 3.59f)
                        reflectiveQuadTo(5f, 3f)
                        horizontalLineTo(17f)
                        quadToRelative(0.82f, 0f, 1.41f, 0.59f)
                        reflectiveQuadTo(19f, 5f)
                        verticalLineToRelative(5.07f)
                        quadTo(18.75f, 10.02f, 18.5f, 10.01f)
                        reflectiveQuadTo(18f, 10f)
                        reflectiveQuadToRelative(-0.5f, 0.01f)
                        reflectiveQuadTo(17f, 10.07f)
                        verticalLineTo(5f)
                        horizontalLineTo(5f)
                        verticalLineTo(15f)
                        horizontalLineToRelative(7.08f)
                        quadToRelative(-0.05f, 0.25f, -0.06f, 0.5f)
                        reflectiveQuadTo(12f, 16f)
                        reflectiveQuadToRelative(0.01f, 0.5f)
                        reflectiveQuadTo(12.08f, 17f)
                        horizontalLineTo(6f)
                        lineTo(3f, 20f)
                        close()
                        moveTo(7f, 9f)
                        horizontalLineToRelative(8f)
                        verticalLineTo(7f)
                        horizontalLineTo(7f)
                        verticalLineTo(9f)
                        close()
                        moveToRelative(0f, 4f)
                        horizontalLineToRelative(5f)
                        verticalLineTo(11f)
                        horizontalLineTo(7f)
                        verticalLineToRelative(2f)
                        close()
                        moveToRelative(10f, 7f)
                        verticalLineTo(17f)
                        horizontalLineTo(14f)
                        verticalLineTo(15f)
                        horizontalLineToRelative(3f)
                        verticalLineTo(12f)
                        horizontalLineToRelative(2f)
                        verticalLineToRelative(3f)
                        horizontalLineToRelative(3f)
                        verticalLineToRelative(2f)
                        horizontalLineTo(19f)
                        verticalLineToRelative(3f)
                        horizontalLineTo(17f)
                        close()
                        moveTo(5f, 15f)
                        verticalLineTo(5f)
                        verticalLineToRelative(5.07f)
                        quadToRelative(0f, 1.25f, 0f, 2.46f)
                        reflectiveQuadTo(5f, 15f)
                        close()
                    }
                }
                .build()
        return _newChatIcon!!
    }

private var _newChatIcon: ImageVector? = null
