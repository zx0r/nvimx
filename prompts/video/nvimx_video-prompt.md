# === 1. Convert macOS screen recording (.mov) → optimized .mp4 ===
# - H.264 for compatibility
# - CRF 23 = good quality/size balance (lower = better quality)
# - preset slow = better compression
# - scale to 1280px width (good for README)
ffmpeg -i nvimx-last.mov -vf "scale=1280:-2" -r 10 -c:v libx264 -preset slow -crf 23 -an nvimx-last.mp4
// ffmpeg -i nvimx-last.mov -vf "scale=960:-2" -r 10 -c:v libx264 -preset slow -crf 24 -an nvimx-last.mp4

# === 2. Create high-quality GIF preview (for README preview) ===
# Step A: generate palette (important for clean colors)
ffmpeg -i nvimx-last.mp4 -vf "fps=12,scale=800:-2:flags=lanczos,format=rgb24,palettegen" -frames:v 1 -update 1 palette.png
// ffmpeg -i nvimx-demo.mp4 -vf "fps=12,scale=800:-1:flags=lanczos,format=rgb24,palettegen" -frames:v 1 -update 1 palette.png

# Step B: create GIF using palette
ffmpeg -i nvimx-last.mp4 -i palette.png -filter_complex "fps=12,scale=800:-2:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer" -loop 0 nvimx-demo.gif
//ffmpeg -i nvimx-demo.mp4 -i palette.png -filter_complex "fps=12,scale=800:-1:flags=lanczos[x];[x][1:v]paletteuse" -loop 0 nvimx-demo.gif

ffmpeg -ss 00:00:06 -t 8 -i nvimx-last.mp4 -i palette.png -filter_complex "fps=12,scale=800:-2:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer" -loop 0 nvimx-demo.gif

# === 3. (Optional) Further compress GIF if too large ===
# Requires: brew install gifsicle
gifsicle -O3 --colors 256 nvimx-demo.gif -o nvimx-demo.min.gif

# === 4. Move to project structure ===
mkdir -p assets/gifs assets/video
mv nvimx-demo.mp4 assets/video/
mv nvimx-demo.min.gif assets/gifs/nvimx-demo.gif

# === 5. Embed in README (GIF preview → click opens HD video) ===
# Copy this snippet into README.md:

# <p align="center">
#   <a href="./assets/video/nvimx-demo.mp4">
#     <img src="./assets/gifs/nvimx-demo.gif" width="720">
#   </a>
# </p>
