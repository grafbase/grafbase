import { readFileSync, promises as fsPromises } from 'fs';
import { join } from 'path';
import fetch from "node-fetch";

function debugPrint(s: string, debug: boolean): void {
    if (debug) {
        console.log(s);
    }
}

function debugPrintArray(s: string[], debug: boolean): void {
    if (debug) {
        console.log(s);
    }
}

// Read file synchronously
function syncReadFile(filename: string): string {
    const result = readFileSync(join(__dirname, filename), 'utf-8');
    return result;
}

function makeCall(content: string): void {
    const results = fetch(endpoint, {
        method: "POST",
        headers: {
            "content-type": "application/json",
            "authorization": "Bearer " + apiKey
        },
        body: content
    })
        .then((res: { text: () => string; }) => res.text())
        .then((text: string) => console.log(text));
};

const debug = process.env.DEBUG != "";

debugPrint("Debugging enabled.", debug);
debugPrint("", debug);

// Get filename specified after -d
const myArgs = process.argv.slice(2);

if (myArgs.length != 2) {
    console.log("Missing -d FILENAME args");
    process.exit();
}

debugPrint("Command line args:", debug);
debugPrintArray(myArgs, debug);
debugPrint("", debug);

const apiKey = process.env.API_KEY;
const endpoint = process.env.ENDPOINT;

debugPrint("Endpoint: " + endpoint, debug);
debugPrint("", debug);
debugPrint("API key: " + apiKey, debug);
debugPrint("", debug);

var filename = "";

if (myArgs[0] == "-d") {
    filename = myArgs[1];
    debugPrint("Reading contents of " + filename, debug);
    debugPrint("", debug);
} else {
    console.log("Missing -d flag");
    process.exit();
}

var result = syncReadFile(filename);

makeCall(result);