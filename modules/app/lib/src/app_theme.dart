import 'package:flutter/material.dart';

class AppPalette {
  static const Color background = Color(0xFFF2F5F9);
  static const Color panel = Color(0xFFFFFFFF);
  static const Color panelMuted = Color(0xFFF7FAFF);
  static const Color line = Color(0xFFE3EAF5);
  static const Color text = Color(0xFF16263D);
  static const Color textSoft = Color(0xFF5A6C84);
  static const Color textMuted = Color(0xFF8797AB);
  static const Color accent = Color(0xFF2F6FD6);
  static const Color accentSoft = Color(0x142F6FD6);
  static const Color up = Color(0xFFEF6477);
  static const Color down = Color(0xFF19B47D);
  static const Color shadow = Color(0x14244068);
}

ThemeData buildAppTheme() {
  const colorScheme = ColorScheme.light(
    primary: AppPalette.accent,
    onPrimary: Colors.white,
    surface: AppPalette.panel,
    onSurface: AppPalette.text,
    error: AppPalette.up,
    onError: Colors.white,
  );

  return ThemeData(
    useMaterial3: true,
    colorScheme: colorScheme,
    scaffoldBackgroundColor: AppPalette.background,
    cardTheme: CardThemeData(
      color: AppPalette.panel,
      elevation: 0,
      margin: EdgeInsets.zero,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(24)),
    ),
    chipTheme: ChipThemeData(
      backgroundColor: AppPalette.accentSoft,
      disabledColor: AppPalette.panelMuted,
      selectedColor: AppPalette.accentSoft,
      secondarySelectedColor: AppPalette.accentSoft,
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      labelStyle: const TextStyle(
        color: AppPalette.accent,
        fontSize: 12,
        fontWeight: FontWeight.w700,
      ),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(999)),
      side: BorderSide.none,
    ),
    textTheme: const TextTheme(
      headlineLarge: TextStyle(
        color: AppPalette.text,
        fontSize: 34,
        fontWeight: FontWeight.w800,
        letterSpacing: -0.6,
      ),
      headlineMedium: TextStyle(
        color: AppPalette.text,
        fontSize: 28,
        fontWeight: FontWeight.w800,
        letterSpacing: -0.4,
      ),
      titleLarge: TextStyle(
        color: AppPalette.text,
        fontSize: 20,
        fontWeight: FontWeight.w800,
      ),
      titleMedium: TextStyle(
        color: AppPalette.text,
        fontSize: 16,
        fontWeight: FontWeight.w700,
      ),
      bodyLarge: TextStyle(color: AppPalette.text, fontSize: 14, height: 1.55),
      bodyMedium: TextStyle(
        color: AppPalette.textSoft,
        fontSize: 13,
        height: 1.6,
      ),
      bodySmall: TextStyle(
        color: AppPalette.textMuted,
        fontSize: 12,
        fontWeight: FontWeight.w600,
      ),
      labelLarge: TextStyle(
        color: AppPalette.text,
        fontSize: 13,
        fontWeight: FontWeight.w700,
      ),
      labelMedium: TextStyle(
        color: AppPalette.textMuted,
        fontSize: 11,
        fontWeight: FontWeight.w700,
        letterSpacing: 0.2,
      ),
    ),
    dividerColor: AppPalette.line,
  );
}
