# Using TypeScript with Graphbase

This file describes the commands to manage a blog using **TypeScript**.
See the parent **README.MD** file for information about
configuring your environment and some GraphQL queries and mutations.

## Getting started

If you are familiar with setting up and configuring a Node/TypeScript project,
you can skip this section.

To jump start your copy of this project:

1. Make sure you have **node** and **npm** installed:

   ```sh
   node -v
   npm -v
   ```

   To install **node** and **npm**, see the [node](https://nodejs.org/en/download/) download page.

2. Create a local directory for the project and navigate into it.
   We use **BlogTest** as a sub-directory of the current directory.

   ```sh
   mkdir BlogTest
   cd BlogTest
   ```

3. Initialize the **BlogTest** project:

   ```sh
   npm init -y
   ```

4. Install TypeScript and the Node types and confirm the TypeScript installation:

   ```sh
   npm install typescript --global
   npm install @types/node --global
   tsc --version
   ```

5. Create a **tsconfig.json** to configure the compiler options for a project:

   ```sh
   touch tsconfig.json
   ```

6. Add the following content to **tsconfig.json**:

   ```json
   {
     "include": ["src"],
     "exclude": ["node_modules"],
     "compilerOptions": {
       "outDir": "dist"
     },
     "lib": ["es2015"]
   }
   ```

7. Since our code uses the **fetch** library,
   install it (we need version 2 as v3 is an ESM-only module):

   ```sh
   npm install node-fetch@2
   ```

## Compiling and Running the Code

The TypeScript source code is in **src/index.ts**.
It takes one argument, 
the name of the JSON file containing the query or mutation.

Enter the following command to compile the TypeScript code to JavaScript:

```sh
npx tsc
```

Enter the following command to execute the resulting JavaScript code,
where *FILENAME.json* is the name of the JSON file containing the query or mutation:

```sh
node dist/index.js -d FILENAME.json
```

You can find a number of pre-defined queries and mutations in the parent folder.
The parent **README.MD** describes each of these JSON files.
