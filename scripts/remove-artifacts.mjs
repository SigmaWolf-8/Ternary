import fs from 'fs';

const OWNER = 'SigmaWolf-8';
const REPO = 'Ternary';
const BRANCH = 'main';
const TOKEN = process.env.GITHUB_TOKEN;

if (!TOKEN) {
  console.error('GITHUB_TOKEN not set');
  process.exit(1);
}

const headers = {
  'Authorization': `Bearer ${TOKEN}`,
  'Accept': 'application/vnd.github.v3+json',
  'Content-Type': 'application/json',
  'User-Agent': 'Salvi-Framework-Sync'
};

async function api(endpoint, method = 'GET', body = null) {
  const opts = { method, headers };
  if (body) opts.body = JSON.stringify(body);
  const res = await fetch(`https://api.github.com/repos/${OWNER}/${REPO}/${endpoint}`, opts);
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(`${method} ${endpoint}: ${res.status} ${err.message || ''}`);
  }
  return res.json();
}

async function main() {
  console.log('=== Task 0-4: Remove Build Artifacts from GitHub ===');
  console.log('Using Git Data API to reconstruct tree without src/kernel/target/\n');

  console.log('Step 1: Get current HEAD commit...');
  const ref = await api(`git/refs/heads/${BRANCH}`);
  const headSha = ref.object.sha;
  console.log(`  HEAD: ${headSha}`);

  console.log('Step 2: Get commit tree...');
  const commit = await api(`git/commits/${headSha}`);
  const treeSha = commit.tree.sha;
  console.log(`  Tree: ${treeSha}`);

  console.log('Step 3: Get full tree recursively...');
  const tree = await api(`git/trees/${treeSha}?recursive=1`);
  console.log(`  Total items: ${tree.tree.length}`);

  const targetItems = tree.tree.filter(item => item.path.startsWith('src/kernel/target/'));
  console.log(`  Items in src/kernel/target/: ${targetItems.length}`);

  if (targetItems.length === 0) {
    console.log('\nNo build artifacts found in src/kernel/target/ — already clean!');
    return;
  }

  const totalSize = targetItems.reduce((sum, item) => sum + (item.size || 0), 0);
  console.log(`  Estimated size: ${(totalSize / 1024 / 1024).toFixed(1)} MB`);

  console.log('\nStep 4: Create new tree without target/ artifacts...');
  const filteredTree = tree.tree
    .filter(item => !item.path.startsWith('src/kernel/target/'))
    .filter(item => item.type === 'blob')
    .map(item => ({
      path: item.path,
      mode: item.mode,
      type: item.type,
      sha: item.sha
    }));

  console.log(`  Filtered items: ${filteredTree.length} (removed ${targetItems.length})`);

  const batchSize = 500;
  let newTreeSha;

  if (filteredTree.length > batchSize) {
    console.log(`  Tree too large for single API call, creating base tree first...`);
    const baseTree = await api('git/trees', 'POST', {
      tree: filteredTree.slice(0, batchSize)
    });
    let currentBase = baseTree.sha;

    for (let i = batchSize; i < filteredTree.length; i += batchSize) {
      const batch = filteredTree.slice(i, i + batchSize);
      console.log(`  Adding batch ${Math.floor(i/batchSize) + 1}... (${batch.length} items)`);
      const nextTree = await api('git/trees', 'POST', {
        base_tree: currentBase,
        tree: batch
      });
      currentBase = nextTree.sha;
    }
    newTreeSha = currentBase;
  } else {
    const newTree = await api('git/trees', 'POST', {
      tree: filteredTree
    });
    newTreeSha = newTree.sha;
  }

  console.log(`  New tree: ${newTreeSha}`);

  console.log('\nStep 5: Create new commit...');
  const newCommit = await api('git/commits', 'POST', {
    message: 'fix(repo): remove 280MB build artifacts from tracking\n\nRemoves 522 compiled Rust artifacts (.rlib, .rmeta, .so, .d files)\nfrom src/kernel/target/ that were tracked in git. These debug build\noutputs should never have been committed to version control.\n\nFinding NF-1 from Third-Party Engineering Review.',
    tree: newTreeSha,
    parents: [headSha]
  });
  console.log(`  New commit: ${newCommit.sha}`);

  console.log('\nStep 6: Update main branch ref...');
  const updated = await api(`git/refs/heads/${BRANCH}`, 'PATCH', {
    sha: newCommit.sha,
    force: true
  });
  console.log(`  Updated: ${updated.object.sha}`);

  console.log('\n=== Build artifacts removed successfully! ===');
  console.log(`Removed ${targetItems.length} files from tracking.`);
}

main().catch(err => {
  console.error('Error:', err.message);
  process.exit(1);
});
