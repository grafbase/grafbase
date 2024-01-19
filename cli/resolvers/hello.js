export default async function Resolver(_, {  }) {
  console.log("test!")
  return await fetch(new URL("https://example.com")).then(response => response.text())
}
