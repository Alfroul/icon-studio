export interface Wallpaper {
  id: string;
  name: string;
  style: string;
}

export const wallpapers: Wallpaper[] = [
  {
    id: "ios-dark",
    name: "iOS Default",
    style: "linear-gradient(135deg, #0a1628 0%, #1a3a5c 40%, #0d2137 100%)",
  },
  {
    id: "ios-light",
    name: "iOS Light",
    style: "linear-gradient(135deg, #b3d9ff 0%, #e0f0ff 50%, #c4dbf0 100%)",
  },
  {
    id: "material-you",
    name: "Material You",
    style: "linear-gradient(135deg, #6750a4 0%, #7f67be 30%, #9a82db 60%, #b8b0e8 100%)",
  },
  {
    id: "dark",
    name: "Dark",
    style: "linear-gradient(135deg, #1a1a1a 0%, #2d2d2d 50%, #1a1a1a 100%)",
  },
  {
    id: "nature",
    name: "Nature",
    style: "linear-gradient(135deg, #134e5e 0%, #2a8a6e 50%, #71b280 100%)",
  },
  {
    id: "sunset",
    name: "Sunset",
    style: "linear-gradient(135deg, #f12711 0%, #f5af19 50%, #f0c27f 100%)",
  },
  {
    id: "white",
    name: "Pure White",
    style: "linear-gradient(135deg, #ffffff 0%, #f5f5f5 100%)",
  },
  {
    id: "black",
    name: "Pure Black",
    style: "linear-gradient(135deg, #000000 0%, #111111 100%)",
  },
];
