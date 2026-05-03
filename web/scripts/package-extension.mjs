import {
  cp,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  stat,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const sourceDir = path.join(root, "out-extension");
const distDir = path.join(root, "extension-dist");
const chromeDir = path.join(distDir, "chrome");
const firefoxDir = path.join(distDir, "firefox");
const safariDir = path.join(distDir, "safari");
const safariAppDir = path.join(distDir, "safari-app");
const safariAppName = "DictDeck Selection Translator";
const safariBundleIdentifier = "com.dictdeck.selection-translator";
const safariDeploymentTarget = "11.0";
const safariBackgroundFile = "background.safari.js";
const launchServicesRegister =
  "/System/Library/Frameworks/CoreServices.framework/Versions/Current/Frameworks/LaunchServices.framework/Versions/Current/Support/lsregister";

async function copyExtension(targetDir) {
  await rm(targetDir, { recursive: true, force: true });
  await mkdir(targetDir, { recursive: true });
  await cp(sourceDir, targetDir, { recursive: true });
}

async function loadSourceManifest() {
  return JSON.parse(
    await readFile(path.join(sourceDir, "manifest.json"), "utf8"),
  );
}

function cloneManifest(manifest) {
  return JSON.parse(JSON.stringify(manifest));
}

function createFirefoxManifest(baseManifest) {
  const manifest = cloneManifest(baseManifest);
  manifest.background = {
    scripts: ["background.js"],
    type: "module",
  };
  manifest.browser_specific_settings = {
    gecko: {
      id: "dictdeck-selection-translator@example.com",
      strict_min_version: "109.0",
    },
  };
  return manifest;
}

function createSafariManifest(baseManifest) {
  const manifest = cloneManifest(baseManifest);
  manifest.background = {
    service_worker: safariBackgroundFile,
  };
  if (manifest.options_ui?.page) {
    manifest.options_ui = {
      page: manifest.options_ui.page,
    };
  }
  return manifest;
}

async function writeManifest(targetDir, manifest) {
  await writeFile(
    path.join(targetDir, "manifest.json"),
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
}

async function assertSafariBackground() {
  try {
    await stat(path.join(sourceDir, safariBackgroundFile));
  } catch {
    throw new Error(
      `Missing Safari background bundle: ${safariBackgroundFile}. Run the Safari-specific Vite build first.`,
    );
  }
}

function commandExists(command, args = ["--version"]) {
  const result = spawnSync(command, args, { stdio: "ignore" });
  return result.status === 0;
}

function findSafariPackagingTool() {
  for (const tool of [
    "safari-web-extension-packager",
    "safari-web-extension-converter",
  ]) {
    const result = spawnSync("xcrun", ["--find", tool], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    });
    if (result.status === 0) {
      return tool;
    }
  }

  return null;
}

function executableExists(command) {
  return !spawnSync(command, [], { stdio: "ignore" }).error;
}

function getSafariSigningConfig() {
  const explicitTeam = process.env.SAFARI_DEVELOPMENT_TEAM;
  const explicitIdentity = process.env.SAFARI_CODE_SIGN_IDENTITY;

  if (explicitTeam || explicitIdentity) {
    return {
      identity: explicitIdentity ?? "Apple Development",
      team: explicitTeam ?? "",
    };
  }

  const result = spawnSync("security", ["find-identity", "-v", "-p", "codesigning"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "ignore"],
  });
  if (result.status !== 0) return null;

  const identity = result.stdout
    .split("\n")
    .map((line) =>
      line.match(
        /"([^"]*(?:Apple Development|Developer ID Application|Mac Developer)[^"]*)"/,
      )?.[1],
    )
    .find(Boolean);
  if (!identity) return null;

  return {
    identity,
    team: identity.match(/\(([A-Z0-9]{10})\)$/)?.[1] ?? "",
  };
}

function hasAppleDevelopmentCertificate() {
  const result = spawnSync("security", ["find-certificate", "-a", "-c", "Apple Development"], {
    stdio: "ignore",
  });
  return result.status === 0;
}

async function findFilesByExtension(dir, extension, depth = 8) {
  if (depth < 0) return [];

  const entries = await readdir(dir, { withFileTypes: true }).catch(() => []);
  const matches = [];

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory() && entry.name.endsWith(extension)) {
      matches.push(fullPath);
      continue;
    }
    if (entry.isDirectory()) {
      matches.push(...(await findFilesByExtension(fullPath, extension, depth - 1)));
    }
  }

  return matches;
}

async function signSafariAppAdhoc(app) {
  if (!executableExists("codesign")) {
    console.warn("codesign not found. Safari app was built, but not explicitly signed.");
    return;
  }

  const extensions = await findFilesByExtension(app, ".appex", 5);
  for (const extension of extensions) {
    const result = spawnSync(
      "codesign",
      ["--force", "--sign", "-", "--timestamp=none", extension],
      { stdio: "inherit" },
    );
    if (result.status !== 0) {
      throw new Error(`Could not sign Safari extension bundle: ${extension}`);
    }
  }

  const result = spawnSync(
    "codesign",
    ["--force", "--sign", "-", "--timestamp=none", "--deep", app],
    { stdio: "inherit" },
  );
  if (result.status !== 0) {
    throw new Error("Could not sign Safari macOS app.");
  }
}

function registerSafariApp(app) {
  const result = spawnSync(launchServicesRegister, ["-f", "-R", "-trusted", app], {
    stdio: "ignore",
  });
  if (result.status !== 0) {
    console.warn(
      "Could not register Safari macOS app with LaunchServices. Open the app once from Finder.",
    );
  }
}

async function registerSafariExtensions(app) {
  if (!executableExists("pluginkit")) return;

  const extensions = await findFilesByExtension(app, ".appex", 5);
  for (const extension of extensions) {
    spawnSync("pluginkit", ["-r", extension], { stdio: "ignore" });
    const result = spawnSync("pluginkit", ["-a", extension], { stdio: "ignore" });
    if (result.status !== 0) {
      console.warn(
        `Could not register Safari extension bundle with pluginkit: ${extension}`,
      );
    }
  }
}

async function buildSafariMacApp(projectDir) {
  if (!commandExists("xcodebuild", ["-version"])) {
    throw new Error(
      "xcodebuild not found. Safari support now requires building the generated macOS app.",
    );
  }

  const buildDir = path.join(projectDir, "build");
  const signing = getSafariSigningConfig();
  const signingArgs = signing
    ? [
        "-allowProvisioningUpdates",
        "CODE_SIGN_STYLE=Automatic",
        `CODE_SIGN_IDENTITY=${signing.identity}`,
        ...(signing.team ? [`DEVELOPMENT_TEAM=${signing.team}`] : []),
      ]
    : ["CODE_SIGNING_ALLOWED=NO"];

  if (signing) {
    console.log(
      `Building signed Safari app with ${signing.identity}${signing.team ? ` / ${signing.team}` : ""}.`,
    );
  } else if (hasAppleDevelopmentCertificate()) {
    console.warn(
      "Apple Development certificate found, but no valid signing identity is available. The matching private key is probably missing from Keychain.",
    );
  } else {
    console.warn(
      "No Apple code signing identity found. Building an ad-hoc Safari app; Safari may still refuse to list it.",
    );
  }

  const result = spawnSync(
    "xcodebuild",
    [
      "-scheme",
      safariAppName,
      "-configuration",
      "Release",
      "-derivedDataPath",
      buildDir,
      `MACOSX_DEPLOYMENT_TARGET=${safariDeploymentTarget}`,
      ...signingArgs,
    ],
    { cwd: projectDir, stdio: "inherit" },
  );

  if (result.status !== 0) {
    throw new Error(
      "Safari macOS app build failed. Inspect the generated Xcode project for signing or scheme issues.",
    );
  }

  const apps = await findFilesByExtension(buildDir, ".app");
  const app = apps.find((value) => value.includes("Release")) ?? apps[0];
  if (!app) {
    throw new Error("Safari macOS app build completed, but no .app bundle was found.");
  }

  if (!signing) {
    await signSafariAppAdhoc(app);
  }
  await rm(safariAppDir, { recursive: true, force: true });
  await mkdir(safariAppDir, { recursive: true });
  const copiedApp = path.join(safariAppDir, path.basename(app));
  await cp(app, copiedApp, { recursive: true });
  if (!signing) {
    await signSafariAppAdhoc(copiedApp);
  }
  registerSafariApp(copiedApp);
  await registerSafariExtensions(copiedApp);
  console.log(`Safari macOS app: ${copiedApp}`);
}

async function fixSafariProjectBundleIdentifiers(projectDir) {
  const projects = await findFilesByExtension(projectDir, ".xcodeproj", 2);
  const pbxproj = projects[0] ? path.join(projects[0], "project.pbxproj") : "";
  if (!pbxproj) {
    throw new Error("No Xcode project found to fix bundle identifiers.");
  }

  const appBundlePattern =
    /PRODUCT_BUNDLE_IDENTIFIER = "com\.dictdeck\.DictDeck-Selection-Translator";/g;
  const deploymentTargetPattern = /MACOSX_DEPLOYMENT_TARGET = 26\.4;/g;
  const content = await readFile(pbxproj, "utf8");
  const fixed = content
    .replace(
      appBundlePattern,
      `PRODUCT_BUNDLE_IDENTIFIER = "${safariBundleIdentifier}";`,
    )
    .replace(
      deploymentTargetPattern,
      `MACOSX_DEPLOYMENT_TARGET = ${safariDeploymentTarget};`,
    );
  if (fixed !== content) {
    await writeFile(pbxproj, fixed);
  }
}

async function buildSafariApp() {
  await assertSafariBackground();

  if (!commandExists("xcrun", ["--version"])) {
    throw new Error(
      "xcrun not found. Safari support now requires Xcode or Command Line Tools.",
    );
  }

  const safariPackagingTool = findSafariPackagingTool();
  if (!safariPackagingTool) {
    throw new Error(
      "Could not find Safari Web Extension packaging tool. Install full Xcode and make sure either safari-web-extension-packager or safari-web-extension-converter is available via xcrun.",
    );
  }

  const tempRoot = await mkdtemp(path.join(tmpdir(), "dictdeck-safari-"));
  const tempExtensionDir = path.join(tempRoot, "extension");
  const tempSafariDir = path.join(tempRoot, "safari");

  try {
    await cp(safariDir, tempExtensionDir, { recursive: true });
    await mkdir(tempSafariDir, { recursive: true });

    const result = spawnSync(
      "xcrun",
      [
        safariPackagingTool,
        tempExtensionDir,
        "--project-location",
        tempSafariDir,
        "--app-name",
        safariAppName,
        "--bundle-identifier",
        safariBundleIdentifier,
        "--copy-resources",
        "--force",
        "--macos-only",
        "--no-open",
        "--no-prompt",
      ],
      { stdio: "inherit" },
    );

    if (result.status !== 0) {
      throw new Error(
        `Safari app generation failed using ${safariPackagingTool}. The Safari package is not considered valid.`,
      );
    }

    await rm(safariDir, { recursive: true, force: true });
    await mkdir(safariDir, { recursive: true });
    await cp(tempSafariDir, safariDir, { recursive: true });

    const projects = await findFilesByExtension(safariDir, ".xcodeproj", 4);
    const project = projects[0];
    if (!project) {
      throw new Error("Safari converter completed, but no Xcode project was found.");
    }

    const projectDir = path.dirname(project);
    await fixSafariProjectBundleIdentifiers(projectDir);
    await buildSafariMacApp(projectDir);
  } finally {
    await rm(tempRoot, { recursive: true, force: true });
  }
}

const sourceManifest = await loadSourceManifest();

await rm(distDir, { recursive: true, force: true });

await copyExtension(chromeDir);
await writeManifest(chromeDir, sourceManifest);

await copyExtension(firefoxDir);
await writeManifest(firefoxDir, createFirefoxManifest(sourceManifest));

await copyExtension(safariDir);
await writeManifest(safariDir, createSafariManifest(sourceManifest));

await buildSafariApp();

console.log(`Chrome extension: ${chromeDir}`);
console.log(`Firefox extension: ${firefoxDir}`);
console.log(`Safari output: ${safariDir}`);
console.log(`Safari app output: ${safariAppDir}`);
