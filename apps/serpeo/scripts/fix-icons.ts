import path from "node:path";
import { globbySync } from "globby";
import sharp from "sharp";
import { mkdirSync } from "node:fs";
const CWD = path.join(
  "./src-tauri/gen/apple/Assets.xcassets/AppIcon.appiconset"
);
const PWD = path.join("./src-tauri/gen/apple/Assets.xcassets");

const getiOSImages = () => {
  const images = globbySync("**/*.png", {
    cwd: CWD,
    // absolute: true,
  });

  images.forEach((image) => {
    const imagePath = path.join(CWD, image);
    console.log(imagePath);
    mkdirSync(`${PWD}/temp`, { recursive: true });
    sharp(imagePath).removeAlpha().toFile(`${PWD}/temp/${image}`);
  });
};

getiOSImages();
