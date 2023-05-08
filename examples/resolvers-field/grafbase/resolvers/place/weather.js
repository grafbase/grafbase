export default function Resolver(root) {
  const { location } = root
  const apiKey = process.env.OPENWEATHER_API_KEY

  if (!location) return null

  return fetch(
    `https://api.openweathermap.org/data/2.5/weather?lat=${location.latitude}&lon=${location.longitude}&units=metric&appid=${apiKey}`
  )
    .then((res) => res.json())
    .then(({ main }) => main.temp)
}
