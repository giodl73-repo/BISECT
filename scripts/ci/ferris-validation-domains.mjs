import { appendFileSync, mkdirSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join } from "node:path";

const [ferris, baseRevision, headRevision, outputDirectory] = process.argv.slice(2);

if (!ferris || !baseRevision || !headRevision || !outputDirectory) {
  throw new Error(
    "usage: ferris-validation-domains.mjs <ferris> <base-revision> <head-revision> <output-directory>",
  );
}

const workspaceId = "giodl73-repo/BISECT";
const manifestPath = "Cargo.toml";
const ownerDomainsPath = ".ferris/owner-validation-domains.json";

mkdirSync(outputDirectory, { recursive: true });

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: process.cwd(),
    encoding: "utf8",
    ...options,
  });
  if (result.error) {
    throw result.error;
  }
  return result;
}

function runFerris(name, inputArguments) {
  const result = run(ferris, [
    "validation-plan",
    "--workspace-id",
    workspaceId,
    "--manifest-path",
    manifestPath,
    "--owner-domains",
    ownerDomainsPath,
    ...inputArguments,
    "--format",
    "json",
  ]);
  if (result.status !== 0) {
    throw new Error(`${name} failed:\n${result.stderr}`);
  }
  const plan = JSON.parse(result.stdout);
  writeFileSync(join(outputDirectory, `${name}.json`), `${JSON.stringify(plan, null, 2)}\n`);
  return plan;
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function parseChanges() {
  const mergeBaseResult = run("git", [
    "merge-base",
    baseRevision,
    headRevision,
  ]);
  if (mergeBaseResult.status !== 0) {
    throw new Error(`git merge-base failed:\n${mergeBaseResult.stderr}`);
  }
  const mergeBase = mergeBaseResult.stdout.trim();
  const result = run("git", [
    "diff",
    "--name-status",
    "-z",
    mergeBase,
    headRevision,
  ], { encoding: null });
  if (result.status !== 0) {
    throw new Error(`git diff failed:\n${result.stderr.toString("utf8")}`);
  }

  const fields = result.stdout
    .toString("utf8")
    .split("\0")
    .filter((field) => field.length > 0);
  const changedPaths = [];
  const deletedPaths = [];

  for (let index = 0; index < fields.length;) {
    const status = fields[index++];
    if (status.startsWith("R") || status.startsWith("C")) {
      const oldPath = fields[index++];
      const newPath = fields[index++];
      if (status.startsWith("R")) {
        deletedPaths.push(oldPath);
      }
      changedPaths.push(newPath);
    } else {
      const path = fields[index++];
      if (status === "D") {
        deletedPaths.push(path);
      } else {
        changedPaths.push(path);
      }
    }
  }

  return {
    base_revision: baseRevision,
    merge_base: mergeBase,
    head_revision: headRevision,
    changed_paths: [...new Set(changedPaths)].sort(),
    deleted_paths: [...new Set(deletedPaths)].sort(),
  };
}

function inputArguments(changes) {
  return [
    ...changes.changed_paths.flatMap((path) => ["--changed-path", path]),
    ...changes.deleted_paths.flatMap((path) => ["--deleted-path", path]),
  ];
}

const changes = parseChanges();
assert(
  changes.changed_paths.length + changes.deleted_paths.length > 0,
  "the selected revision range contains no changed paths",
);
writeFileSync(
  join(outputDirectory, "actual-changes.json"),
  `${JSON.stringify(changes, null, 2)}\n`,
);

const actualPlan = runFerris("actual-plan", inputArguments(changes));
const expectedWebSelection = [...changes.changed_paths, ...changes.deleted_paths].some(
  (path) => path === "web/docs" || path.startsWith("web/docs/"),
);
const actualWebSelection = (actualPlan.record.selected_owner_entrypoints ?? []).includes(
  "web-docs-build",
);
assert(
  actualWebSelection === expectedWebSelection,
  "actual owner-domain selection does not match the changed path set",
);

const webOnly = runFerris("scenario-web-only", [
  "--changed-path",
  "web/docs/package.json",
]);
assert(
  webOnly.record.inputs[0].disposition === "owner_domain_path" &&
    webOnly.record.selected_owner_entrypoints.includes("web-docs-build") &&
    !webOnly.record.fallback.required_by_inputs,
  "web-only scenario did not select only the declared owner domain",
);

const mixed = runFerris("scenario-mixed-cargo-web", [
  "--changed-path",
  "crates/bisect-core/src/lib.rs",
  "--changed-path",
  "web/docs/package.json",
]);
assert(
  mixed.record.selected_packages.length > 0 &&
    mixed.record.inputs.some(
      (input) =>
        input.value === "crates/bisect-core/src/lib.rs" &&
        input.disposition === "owned_rust_path",
    ) &&
    mixed.record.selected_packages.some(
      (selection) => selection.package.name === "bisect-core",
    ) &&
    mixed.record.selected_owner_entrypoints.includes("web-docs-build") &&
    !mixed.record.fallback.required_by_inputs,
  "mixed Cargo/web scenario did not preserve both selections",
);

const deletedWeb = runFerris("scenario-deleted-web", [
  "--deleted-path",
  "web/docs/deleted-page.md",
]);
assert(
  deletedWeb.record.inputs[0].disposition === "owner_domain_path" &&
    deletedWeb.record.inputs[0].path_evidence === "lexical_missing" &&
    deletedWeb.record.selected_owner_entrypoints.includes("web-docs-build") &&
    !deletedWeb.record.fallback.required_by_inputs,
  "deleted web scenario did not retain lexical owner-domain evidence",
);

const unknown = runFerris("scenario-unknown-fallback", [
  "--changed-path",
  ".github/workflows/ci.yml",
]);
assert(
  unknown.record.inputs[0].disposition === "full_workspace_fallback" &&
    !(unknown.record.selected_owner_entrypoints ?? []).includes("web-docs-build") &&
    unknown.record.fallback.required_by_inputs,
  "undeclared path did not retain full-workspace fallback",
);

const boundary = runFerris("scenario-prefix-boundary", [
  "--deleted-path",
  "web/docs-legacy/index.md",
]);
assert(
  boundary.record.inputs[0].disposition === "full_workspace_fallback" &&
    !(boundary.record.selected_owner_entrypoints ?? []).includes("web-docs-build") &&
    boundary.record.fallback.required_by_inputs,
  "owner-domain prefix matched across a path-segment boundary",
);

const invalidContractPath = join(outputDirectory, "invalid-overlap-owner-domains.json");
writeFileSync(
  invalidContractPath,
  `${JSON.stringify(
    {
      schema: "ferris.owner-validation-domains/v1",
      workspace_id: workspaceId,
      domains: [
        {
          domain_id: "web-docs",
          path_prefix: "web/docs",
          entrypoint_ids: ["web-docs-build"],
        },
        {
          domain_id: "web-docs-source",
          path_prefix: "web/docs/src",
          entrypoint_ids: ["web-docs-source-build"],
        },
      ],
    },
    null,
    2,
  )}\n`,
);
const invalid = run(ferris, [
  "validation-plan",
  "--workspace-id",
  workspaceId,
  "--manifest-path",
  manifestPath,
  "--owner-domains",
  invalidContractPath,
  "--changed-path",
  "web/docs/package.json",
  "--format",
  "json",
]);
assert(invalid.status !== 0, "overlapping owner-domain prefixes were accepted");
const invalidEnvelope = JSON.parse(invalid.stderr);
assert(
  invalidEnvelope.diagnostics[0].code === "FERRIS-OWNER-DOMAINS-PREFIX-OVERLAP",
  "overlap control failed with the wrong diagnostic",
);
writeFileSync(
  join(outputDirectory, "scenario-invalid-overlap.json"),
  `${JSON.stringify(invalidEnvelope, null, 2)}\n`,
);

if (process.env.GITHUB_OUTPUT) {
  appendFileSync(
    process.env.GITHUB_OUTPUT,
    `web_docs_selected=${actualWebSelection ? "true" : "false"}\n`,
  );
}
