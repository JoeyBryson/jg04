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
public val personAddIcon: ImageVector
    get() {
        if (_personAddIcon != null) {
            return _personAddIcon!!
        }
        _personAddIcon =
            ImageVector.Builder(
                name = "person_add",
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
                        moveTo(18f, 14f)
                        verticalLineTo(11f)
                        horizontalLineTo(15f)
                        verticalLineTo(9f)
                        horizontalLineToRelative(3f)
                        verticalLineTo(6f)
                        horizontalLineToRelative(2f)
                        verticalLineTo(9f)
                        horizontalLineToRelative(3f)
                        verticalLineToRelative(2f)
                        horizontalLineTo(20f)
                        verticalLineToRelative(3f)
                        horizontalLineTo(18f)
                        close()
                        moveTo(6.18f, 10.83f)
                        quadTo(5f, 9.65f, 5f, 8f)
                        reflectiveQuadTo(6.18f, 5.18f)
                        reflectiveQuadTo(9f, 4f)
                        reflectiveQuadToRelative(2.83f, 1.18f)
                        reflectiveQuadTo(13f, 8f)
                        reflectiveQuadToRelative(-1.17f, 2.82f)
                        reflectiveQuadTo(9f, 12f)
                        reflectiveQuadTo(6.18f, 10.83f)
                        close()
                        moveTo(1f, 20f)
                        verticalLineTo(17.2f)
                        quadTo(1f, 16.35f, 1.44f, 15.64f)
                        quadTo(1.88f, 14.93f, 2.6f, 14.55f)
                        quadTo(4.15f, 13.77f, 5.75f, 13.39f)
                        reflectiveQuadTo(9f, 13f)
                        reflectiveQuadToRelative(3.25f, 0.39f)
                        reflectiveQuadToRelative(3.15f, 1.16f)
                        quadToRelative(0.72f, 0.38f, 1.16f, 1.09f)
                        reflectiveQuadTo(17f, 17.2f)
                        verticalLineTo(20f)
                        horizontalLineTo(1f)
                        close()
                        moveTo(3f, 18f)
                        horizontalLineTo(15f)
                        verticalLineTo(17.2f)
                        quadToRelative(0f, -0.27f, -0.14f, -0.5f)
                        quadTo(14.73f, 16.48f, 14.5f, 16.35f)
                        quadTo(13.15f, 15.68f, 11.78f, 15.34f)
                        reflectiveQuadTo(9f, 15f)
                        reflectiveQuadTo(6.23f, 15.34f)
                        reflectiveQuadTo(3.5f, 16.35f)
                        quadTo(3.28f, 16.48f, 3.14f, 16.7f)
                        quadTo(3f, 16.93f, 3f, 17.2f)
                        verticalLineTo(18f)
                        close()
                        moveTo(10.41f, 9.41f)
                        quadTo(11f, 8.82f, 11f, 8f)
                        reflectiveQuadTo(10.41f, 6.59f)
                        reflectiveQuadTo(9f, 6f)
                        quadTo(8.18f, 6f, 7.59f, 6.59f)
                        quadTo(7f, 7.18f, 7f, 8f)
                        reflectiveQuadTo(7.59f, 9.41f)
                        reflectiveQuadTo(9f, 10f)
                        quadToRelative(0.83f, 0f, 1.41f, -0.59f)
                        close()
                        moveTo(9f, 8f)
                        close()
                        moveTo(9f, 18f)
                        close()
                    }
                }
                .build()
        return _personAddIcon!!
    }

private var _personAddIcon: ImageVector? = null
