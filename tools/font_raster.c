/* Offline-only font authoring host. No FreeType dependency enters the game. */
#include <ft2build.h>
#include FT_FREETYPE_H
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char **argv) {
    FT_Library library; FT_Face face;
    if (argc != 3 || FT_Init_FreeType(&library) || FT_New_Face(library, argv[1], 0, &face) ||
        FT_Set_Pixel_Sizes(face, 0, 48)) return 1;
    unsigned char *atlas = calloc(1024 * 384, 1);
    if (!atlas) return 2;
    int major, minor, patch;
    FT_Library_Version(library, &major, &minor, &patch);
    printf("{\"freetype\":\"%d.%d.%d\",\"em\":48,\"width\":1024,\"height\":384,\"glyphs\":[", major, minor, patch);
    for (unsigned int c = 32; c <= 126; c++) {
        if (!FT_Get_Char_Index(face, c) || FT_Load_Char(face, c, FT_LOAD_RENDER | FT_LOAD_NO_HINTING | FT_LOAD_NO_AUTOHINT)) return 3;
        FT_GlyphSlot g = face->glyph; FT_Bitmap b = g->bitmap;
        unsigned int x = ((c - 32) % 16) * 64 + 2, y = ((c - 32) / 16) * 64 + 2;
        if (b.width > 60 || b.rows > 60 || (b.width && (b.pixel_mode != FT_PIXEL_MODE_GRAY || b.num_grays != 256 || b.pitch < (int)b.width))) return 4;
        for (unsigned int row = 0; row < b.rows; row++) memcpy(atlas + (y + row) * 1024 + x, b.buffer + row * b.pitch, b.width);
        printf("%s[%u,%u,%u,%u,%d,%d,%ld]", c == 32 ? "" : ",", x, y, b.width, b.rows, g->bitmap_left, g->bitmap_top, g->advance.x);
    }
    puts("]}"); FILE *out = fopen(argv[2], "wb");
    if (!out || fwrite(atlas, 1, 1024 * 384, out) != 1024 * 384 || fclose(out)) return 5;
    free(atlas); FT_Done_Face(face); FT_Done_FreeType(library); return 0;
}
