import { appendFileSync, mkdirSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join } from "node:path";

const [ferris, baseRevision, headRevision, testedRevision, outputDirectory] =
  process.argv.slice(2);

if (!ferris || !baseRevision || !headRevision || !testedRevision || !outputDirectory) {
  throw new Error(
    "usage: ferris-validation-domains.mjs <ferris> <base-revision> <head-revision> <tested-revision> <output-directory>",
  );
}

const workspaceId = "giodl73-repo/BISECT";
const manifestPath = "Cargo.toml";
const ownerDomainsPath = ".ferris/owner-validation-domains.json";
const maxCommandOutputBytes = 32 * 1024 * 1024;

mkdirSync(outputDirectory, { recursive: true });

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: process.cwd(),
    encoding: "utf8",
    maxBuffer: maxCommandOutputBytes,
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

function hasWebDocsChanges() {
  const result = run("git", [
    "diff",
    "--quiet",
    `${baseRevision}...${headRevision}`,
    "--",
    "web/docs",
  ]);
  if (result.status === 0) {
    return false;
  }
  if (result.status === 1) {
    return true;
  }
  throw new Error(`web/docs change oracle failed:\n${result.stderr}`);
}

const actualPlan = runFerris("actual-plan", [
  "--base-revision",
  baseRevision,
  "--head-revision",
  headRevision,
  "--tested-revision",
  testedRevision,
]);
const revisionBinding = actualPlan.record.revision_binding;
assert(revisionBinding, "actual plan did not include a revision binding");
assert(
  revisionBinding.changed_path_count + revisionBinding.deleted_path_count > 0,
  "the selected revision range contains no changed paths",
);
const expectedWebSelection = hasWebDocsChanges();
const actualWebSelection = (actualPlan.record.selected_owner_entrypoints ?? []).includes(
  "web-docs-build",
);
assert(
  actualWebSelection === expectedWebSelection,
  "actual owner-domain selection does not match the independent web/docs oracle",
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
