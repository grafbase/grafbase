# Using JavaScript with Graphbase

This file describes the commands to manage a blog using **JavaScript**.
See the parent **README.MD** file for information about
configuring your environment and some GraphQL queries and mutations.

## Getting started

If you are familiar with setting up and configuring a Node/JavaScript project,
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

4. Since our code uses the **fetch** library,
   install it:

   ```sh
   npm install node-fetch
   ```

## Using Queries and Mutations from a JSON File

The main program, in **src/index.mjs**,
takes one argument, the name of a JSON file containing the query or mutation.
The command line is like the following,
where *FILENAME.json* is the name of the JSON file
containing the query or mutation:

```sh
node src/index.mjs -d FILENAME.json
```

You can find a number of pre-defined queries and mutations in the parent folder.
The parent **README.MD** describes each of these JSON files.