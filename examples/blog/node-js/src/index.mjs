import fetch from "node-fetch";
import fs from "fs";

var apiKey = process.env.API_KEY;
var endpoint = process.env.ENDPOINT;
var debug = process.env.DEBUG;

// Get filename specified after -d
const myArgs = process.argv.slice(2);

if (debug) {
  console.log('myArgs: ', myArgs);
}

if (myArgs.length != 2) {
  console.log("Missing -d FILENAME args");
  process.exit();
}

if (myArgs[0] == "-d") {
  var filename = myArgs[1];
  if (debug) {
    console.log("Reading contents of " + filename);
  }
} else {
  console.log("Missing -d flag");
  process.exit();
}

fs.readFile(filename, 'utf8', (err, content) => {
  if (err) {
    console.error(err);
    return;
  }

  if (debug) {
    console.log("Contents of " + filename + ":");
    console.log(content);
  }

  const results = fetch(endpoint, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      "authorization": "Bearer " + apiKey
    },
    body: content
  })
    .then(res => res.text())
    .then(text => console.log(text));
});
