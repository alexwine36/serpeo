// @ts-check

const fs = require('node:fs');
const YAML = require('yaml');

let workspaceFile;
let catalogDeps = [];
try {
  workspaceFile = fs.readFileSync('./pnpm-workspace.yaml', 'utf8');
  const parsedWorkspace = YAML.parse(workspaceFile);
  catalogDeps = parsedWorkspace.catalog
    ? Object.keys(parsedWorkspace.catalog)
    : [];

  if (!parsedWorkspace.catalog) {
    console.error(
      'Warning: "catalog" property is missing in the workspace file.'
    );
  }
} catch (error) {
  console.error('Failed to read pnpm-workspace.yaml:', error.message);
  process.exit(1);
}
console.log('Catalog dependencies:', catalogDeps);

/** @type {import("syncpack").RcFile} */
const config = {
  lintSemverRanges: true,
  lintVersions: true,
  lintFormatting: false,
  sortFirst: [
    'name',
    'description',
    'version',
    'private',
    'author',
    'license',
    'scripts',
    'main',
    'module',
    'types',
    'files',
    'dependencies',
    'devDependencies',
  ],
  semverGroups: [
    {
      label: 'use catalog versions when available',
      packages: ['**'],
      dependencies: catalogDeps,
      isIgnored: true,
    },
    {
      label: 'use exact version numbers in production',
      packages: ['**'],
      dependencyTypes: ['prod'],
      dependencies: ['**'],
      // specifierTypes: ["!latest"],
      range: '',
    },
    {
      label: 'use range in development',
      packages: ['**'],
      dependencyTypes: ['dev'],
      dependencies: ['**'],
      range: '^',
    },
  ],
  versionGroups: [
    // {
    //   label: "don't use latest in production",
    //   // packages: ["**"],
    //   // dependencyTypes: ["prod", "dev"],
    //   dependencies: ["**"],
    //   // specifierTypes: ["latest"],
    //   policy: "sameRange",
    //   // range: "^",
    // },
    // {
    //   // dependencies: ["**"],
    //   specifierTypes: ["latest"],
    //   isBanned: true,
    // },
    {
      dependencies: ['@types/**'],
      dependencyTypes: ['!dev'],
      isBanned: true,
      label: '@types packages should only be under devDependencies',
    },
    {
      label: 'Use catalog versions when available',
      dependencies: catalogDeps,
      pinVersion: 'catalog:',
    },
    {
      label: 'Use workspace protocol when developing local packages',
      dependencies: ['$LOCAL'],
      dependencyTypes: ['dev', 'prod', 'peer'],
      pinVersion: 'workspace:*',
    },
  ],
};

module.exports = config;
