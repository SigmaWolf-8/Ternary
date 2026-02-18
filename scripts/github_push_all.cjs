const fs = require('fs');
const path = require('path');

const OWNER = 'SigmaWolf-8';
const REPO = 'Ternary';
const TOKEN = process.env.GITHUB_TOKEN;
const COMMIT_MSG = 'XPlenum Phases 1-7: Complete RISC-V security extension with FIPS 140-3 compliance';

if (!TOKEN) {
  console.error('GITHUB_TOKEN not set');
  process.exit(1);
}

const FILES_TO_PUSH = [
  'docs/xplenum/phase1_core_selection_2026-02-18.md',
  'docs/xplenum/phase1_interface_specification_2026-02-18.md',
  'docs/xplenum/phase1_pipeline_analysis_2026-02-18.md',
  'docs/xplenum/phase4_drbg_algorithm_selection_2026-02-18.md',
  'docs/xplenum/phase6_emulation_validation_report_2026-02-18.md',
  'docs/xplenum/phase7_audit_coordination_package_2026-02-18.md',
  'docs/xplenum/phase7_cnsa2_compliance_2026-02-18.md',
  'docs/xplenum/phase7_fips140_3_compliance_mapping_2026-02-18.md',
  'docs/xplenum/phase7_isa_specification_v1_2026-02-18.md',
  'docs/xplenum/xplenum_completion_task_list_2026-02-18.md',
  '.github/workflows/xplenum-riscv.yml',
  'rtl/formal/xplenum_formal_induction.sby',
  'rtl/formal/xplenum_formal_props.v',
  'rtl/formal/xplenum_formal.sby',
  'rtl/formal/xplenum_induction_helpers.v',
  'rtl/formal/xplenum_integration_formal_props.v',
  'rtl/formal/xplenum_integration_formal.sby',
  'rtl/integration/xplenum_cva6_top.v',
  'rtl/integration/xplenum_cva6_wrapper.v',
  'rtl/integration/xplenum_stall_controller.v',
  'rtl/xplenum_aes256_core.v',
  'rtl/xplenum_cap_unit.v',
  'rtl/xplenum_ctr_drbg.v',
  'rtl/xplenum_domain_unit.v',
  'rtl/xplenum_mask_unit.v',
  'rtl/xplenum_pkg.vh',
  'rtl/xplenum_top.v',
  'rtl/xplenum_trit_unit.v',
  'sim/cross-verify/xplenum_cross_verify.cjs',
  'sim/cross-verify/xplenum_cross_verify.py',
  'sim/fuzzing/Makefile',
  'sim/fuzzing/xplenum_fuzz_harness.cpp',
  'sim/qemu/xplenum_boot_test.sh',
  'sim/qemu/xplenum_e2e_security_tests.py',
  'sim/qemu/xplenum_qemu_helper.c',
  'sim/qemu/xplenum_qemu_trans.c.inc',
  'sim/spike/Makefile',
  'sim/spike/xplenum_spike_extension.h',
  'sim/spike/xplenum_spike_test.cpp',
  'src/kernel/src/arch/xplenum.rs',
  'src/kernel/src/security/xplenum_hal.rs',
  'src/kernel/src/security/xplenum_tests.rs',
  'synth/xplenum_fpga.sdc',
  'synth/xplenum_pinmap.xdc',
  'synth/xplenum_synth.tcl',
  'tb/xplenum_cva6_integration_tb.v',
  'tb/xplenum_drbg_tb.v',
  'tb/xplenum_tb.v',
];

async function githubAPI(endpoint, method = 'GET', body = null) {
  const url = `https://api.github.com/repos/${OWNER}/${REPO}${endpoint}`;
  const headers = {
    'Authorization': `Bearer ${TOKEN}`,
    'Accept': 'application/vnd.github.v3+json',
    'Content-Type': 'application/json',
    'User-Agent': 'XPlenum-Push-Script'
  };
  const opts = { method, headers };
  if (body) opts.body = JSON.stringify(body);
  const resp = await fetch(url, opts);
  return { status: resp.status, data: await resp.json().catch(() => ({})) };
}

async function pushFile(filePath) {
  const fullPath = path.resolve(process.cwd(), filePath);
  if (!fs.existsSync(fullPath)) {
    return { file: filePath, status: 'skip', error: 'File not found' };
  }

  const content = fs.readFileSync(fullPath);
  const encoded = content.toString('base64');

  const existing = await githubAPI(`/contents/${filePath}`);
  const sha = existing.status === 200 ? existing.data.sha : undefined;

  const body = {
    message: `${COMMIT_MSG} - ${filePath}`,
    content: encoded,
  };
  if (sha) body.sha = sha;

  const result = await githubAPI(`/contents/${filePath}`, 'PUT', body);
  if (result.status === 200 || result.status === 201) {
    return { file: filePath, status: 'success' };
  } else {
    return { file: filePath, status: 'error', error: result.data.message || `HTTP ${result.status}` };
  }
}

async function main() {
  console.log(`Pushing ${FILES_TO_PUSH.length} files to ${OWNER}/${REPO}...`);
  console.log('='.repeat(60));

  let succeeded = 0;
  let failed = 0;
  let skipped = 0;

  for (let i = 0; i < FILES_TO_PUSH.length; i++) {
    const file = FILES_TO_PUSH[i];
    process.stdout.write(`[${i+1}/${FILES_TO_PUSH.length}] ${file}... `);

    try {
      const result = await pushFile(file);
      if (result.status === 'success') {
        console.log('OK');
        succeeded++;
      } else if (result.status === 'skip') {
        console.log(`SKIP (${result.error})`);
        skipped++;
      } else {
        console.log(`FAIL (${result.error})`);
        failed++;
      }
    } catch (err) {
      console.log(`ERROR (${err.message})`);
      failed++;
    }

    if (i < FILES_TO_PUSH.length - 1) {
      await new Promise(r => setTimeout(r, 500));
    }
  }

  console.log('='.repeat(60));
  console.log(`Done: ${succeeded} pushed, ${failed} failed, ${skipped} skipped`);
  process.exit(failed > 0 ? 1 : 0);
}

main();
