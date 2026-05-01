# Resize to Open Graph resolution (most effective size reduction)

# Quantize colors (reduce palette, huge size gain, minimal visual loss)
pngquant --quality=70-90 --speed 1 --force --output temp.png resized.png

# Lossless compression + strip metadata (final cleanup)
oxipng -o max --strip all temp.png

# (Optional) Maximum compression (slower, slightly smaller)
oxipng -o max --strip all --zopfli temp.png

# ImageMagick (best control)
magick nvimx-og-banner.png -resize 1280x640 output-magic.png

# macOS built-in alternative (keeps aspect ratio, max width 1280)
sips -Z 1280 nvimx-og-banner.png --out output.png

# Check final size (must be < 1MB)
ls -lh *.png
