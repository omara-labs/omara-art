import os
import math
from PIL import Image, ImageDraw

def main():
    # Paths
    raw_logo_path = "/home/jeryd/.gemini/antigravity-cli/brain/7062a52c-057c-45fe-a542-13a144dba460/omara_logo_raw_1779606440791.png"
    output_dir = "/home/jeryd/Projects/omara-labs/omara-art/screensavers/target/plymouth-build"
    
    os.makedirs(output_dir, exist_ok=True)
    
    # 1. Process Watermark (Omara Labs Logo)
    print("Processing logo watermark...")
    if os.path.exists(raw_logo_path):
        img_logo = Image.open(raw_logo_path)
        # Resize to 320x320 with high-quality resampling
        img_watermark = img_logo.resize((320, 320), Image.Resampling.LANCZOS)
        img_watermark.save(os.path.join(output_dir, "watermark.png"), "PNG")
        print("watermark.png saved successfully.")
    else:
        print(f"Error: Raw logo not found at {raw_logo_path}")
        return

    # 2. Generate 30 Throbber Frames (comet tail spinner)
    print("Generating throbber frames...")
    frames_count = 30
    size = 128
    radius = 35
    center = size // 2
    
    for i in range(frames_count):
        # Create a transparent RGBA image
        frame = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        draw = ImageDraw.Draw(frame)
        
        # Base angle for current frame
        theta = i * (2.0 * math.pi / frames_count)
        
        # Draw 15 trail segments
        trail_length = 15
        for j in range(trail_length):
            # Compute angle offset for tail parts
            angle_offset = -j * (2.0 * math.pi / 45.0) # tail extends about 120 degrees
            angle = theta + angle_offset
            
            # Compute center of the dot
            cx = center + radius * math.cos(angle)
            cy = center + radius * math.sin(angle)
            
            # Fading radius: 4.5 down to 1.0
            r_dot = 4.5 - j * (3.5 / (trail_length - 1))
            
            # Color interpolation: Cyan (0, 255, 255) to Violet (160, 0, 255)
            pct = j / (trail_length - 1)
            r_val = int(0 * (1.0 - pct) + 160 * pct)
            g_val = int(255 * (1.0 - pct) + 0 * pct)
            b_val = int(255 * (1.0 - pct) + 255 * pct)
            
            # Fading opacity
            alpha = int(255 * (1.0 - j / trail_length))
            
            # Draw dot
            draw.ellipse(
                [cx - r_dot, cy - r_dot, cx + r_dot, cy + r_dot],
                fill=(r_val, g_val, b_val, alpha)
            )
            
        # Save frame
        frame_name = f"throbber-{i+1:04d}.png"
        frame.save(os.path.join(output_dir, frame_name), "PNG")
        
    print(f"Generated {frames_count} throbber frames in {output_dir}.")

if __name__ == "__main__":
    main()
