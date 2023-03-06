import "dotenv/config";

export default {
  expo: {
    name: "expo-react-native",
    slug: "expo-react-native",
    version: "1.0.0",
    orientation: "portrait",
    icon: "./assets/icon.png",
    userInterfaceStyle: "light",
    splash: {
      image: "./assets/splash.png",
      resizeMode: "contain",
      backgroundColor: "#000000",
    },
    updates: {
      fallbackToCacheTimeout: 0,
      url: "https://u.expo.dev/8bc839e1-4282-4036-8191-ae54134fe827",
    },
    assetBundlePatterns: ["**/*"],
    ios: {
      bundleIdentifier: "expo.react.native",
      supportsTablet: true,
    },
    android: {
      package: "expo.react.native",
      adaptiveIcon: {
        foregroundImage: "./assets/adaptive-icon.png",
        backgroundColor: "#FFFFFF",
      },
    },
    web: {
      favicon: "./assets/favicon.png",
    },
    runtimeVersion: {
      policy: "sdkVersion",
    },
  },
};
