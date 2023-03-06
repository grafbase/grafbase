require("dotenv/config");

module.exports = function (api) {
  api.cache(true);
  return {
    presets: ["babel-preset-expo"],
    plugins: ["nativewind/babel", "transform-inline-environment-variables"],
  };
};
