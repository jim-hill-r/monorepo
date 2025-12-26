import * as vscode from "vscode";
import fs from "fs";

let myStatusBarItem: vscode.StatusBarItem;

const SESSIONS_DIRECTORY = ".cast/sessions";

export function activate({ subscriptions }: vscode.ExtensionContext) {
  myStatusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100
  );
  subscriptions.push(myStatusBarItem);

  updateStatusBarItem();
}

export function deactivate() {}

function updateStatusBarItem(): void {
  if (
    vscode.workspace.workspaceFolders !== undefined &&
    vscode.workspace.workspaceFolders.length > 0
  ) {
    console.log(vscode.workspace.workspaceFolders);
    const sessionDirectory = `${vscode.workspace.workspaceFolders[0].uri.path}/${SESSIONS_DIRECTORY}`;
    fs.readdir(sessionDirectory, (err, files) => {
      if (!err && files.length > 0) {
        const sortedFiles = files.sort();
        const sessionLogFilePath = `${sessionDirectory}/${sortedFiles[sortedFiles.length - 1]}`;
        const sessionLogContent = fs.readFileSync(sessionLogFilePath, "utf-8");
        const sessionStart = getSessionStart(sessionLogContent);
        if (sessionStart) {
          const elapsed = Math.floor(
            (Date.now() - sessionStart.getTime()) / 1000
          );
          const hours = Math.floor(elapsed / 3600);
          const minutes = Math.floor((elapsed % 3600) / 60);
          const seconds = elapsed % 60;
          myStatusBarItem.text = `${String(hours).padStart(2, "0")}:${String(
            minutes
          ).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
        }
      }
    });

    myStatusBarItem.show();
  } else {
    myStatusBarItem.text = `00:00:00`;
    myStatusBarItem.hide();
  }
  setTimeout(() => updateStatusBarItem(), 1000);
}

export function getSessionStart(sessionLog: string): Date | undefined {
  const lines = sessionLog.split("\n");
  for (let line of lines) {
    const [date, type] = line.split(",");
    if (type === "Start") {
      return new Date(Date.parse(date));
    }
  }
  return undefined;
}
