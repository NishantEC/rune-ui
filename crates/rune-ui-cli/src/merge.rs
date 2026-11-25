//! Line diffs and the three-way merge `update` uses: our edits and the registry's changes since the copy we made, combined, with conflict
//! markers only where both changed the same lines differently.

/// Pairs of (a index, b index) on a longest common subsequence of lines.
fn lcs(a: &[&str], b: &[&str]) -> Vec<(usize, usize)> {
    let (n, m) = (a.len(), b.len());
    let mut t = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            t[i][j] = if a[i] == b[j] { t[i + 1][j + 1] + 1 } else { t[i + 1][j].max(t[i][j + 1]) };
        }
    }
    let (mut i, mut j, mut out) = (0, 0, Vec::new());
    while i < n && j < m {
        if a[i] == b[j] {
            out.push((i, j));
            i += 1;
            j += 1;
        } else if t[i + 1][j] >= t[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

/// A unified-style diff of `a` (theirs, `---`) against `b` (yours, `+++`), with `context` lines round each change.
pub fn unified(a: &str, b: &str, name_a: &str, name_b: &str, context: usize) -> String {
    let (al, bl): (Vec<&str>, Vec<&str>) = (a.lines().collect(), b.lines().collect());
    let pairs = lcs(&al, &bl);
    let mut ops: Vec<(char, &str)> = Vec::new();
    let (mut i, mut j) = (0, 0);
    for &(pi, pj) in pairs.iter().chain(std::iter::once(&(al.len(), bl.len()))) {
        while i < pi {
            ops.push(('-', al[i]));
            i += 1;
        }
        while j < pj {
            ops.push(('+', bl[j]));
            j += 1;
        }
        if pi < al.len() {
            ops.push((' ', al[pi]));
            i += 1;
            j += 1;
        }
    }
    if ops.iter().all(|(c, _)| *c == ' ') {
        return String::new();
    }
    let mut out = format!("--- {name_a}\n+++ {name_b}\n");
    let changed: Vec<usize> = ops.iter().enumerate().filter(|(_, (c, _))| *c != ' ').map(|(k, _)| k).collect();
    let mut shown = vec![false; ops.len()];
    for k in changed {
        for s in shown.iter_mut().take((k + context + 1).min(ops.len())).skip(k.saturating_sub(context)) {
            *s = true;
        }
    }
    let mut gap = false;
    for (k, (c, l)) in ops.iter().enumerate() {
        if shown[k] {
            if gap {
                out.push_str("@@\n");
                gap = false;
            }
            out.push_str(&format!("{c}{l}\n"));
        } else {
            gap = true;
        }
    }
    out
}

/// The merge of `ours` and `theirs`, both changed from `base`, and whether it has conflicts.
pub fn three_way(base: &str, ours: &str, theirs: &str) -> (String, bool) {
    let (b, o, t): (Vec<&str>, Vec<&str>, Vec<&str>) = (base.lines().collect(), ours.lines().collect(), theirs.lines().collect());
    let bo: std::collections::HashMap<usize, usize> = lcs(&b, &o).into_iter().collect();
    let bt: std::collections::HashMap<usize, usize> = lcs(&b, &t).into_iter().collect();
    // Anchors: base lines kept by both sides.
    let mut anchors: Vec<(usize, usize, usize)> = (0..b.len()).filter_map(|i| Some((i, *bo.get(&i)?, *bt.get(&i)?))).collect();
    anchors.push((b.len(), o.len(), t.len()));
    let (mut pb, mut po, mut pt) = (0, 0, 0);
    let mut out: Vec<String> = Vec::new();
    let mut conflict = false;
    for (ab, ao, at) in anchors {
        let (cb, co, ct) = (&b[pb..ab], &o[po..ao], &t[pt..at]);
        if co == cb {
            out.extend(ct.iter().map(|s| (*s).to_owned()));
        } else if ct == cb || co == ct {
            out.extend(co.iter().map(|s| (*s).to_owned()));
        } else {
            conflict = true;
            out.push("<<<<<<< yours".to_owned());
            out.extend(co.iter().map(|s| (*s).to_owned()));
            out.push("=======".to_owned());
            out.extend(ct.iter().map(|s| (*s).to_owned()));
            out.push(">>>>>>> rune-ui".to_owned());
        }
        if ab < b.len() {
            out.push(b[ab].to_owned());
        }
        (pb, po, pt) = (ab + 1, ao + 1, at + 1);
    }
    let mut s = out.join("\n");
    if theirs.ends_with('\n') || ours.ends_with('\n') {
        s.push('\n');
    }
    (s, conflict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separate_edits_merge_cleanly() {
        let base = "a\nb\nc\nd\n";
        let ours = "a\nB (mine)\nc\nd\n";
        let theirs = "a\nb\nc\nd\ne (new upstream)\n";
        assert_eq!(three_way(base, ours, theirs), ("a\nB (mine)\nc\nd\ne (new upstream)\n".to_owned(), false));
    }

    #[test]
    fn the_same_lines_changed_twice_conflict() {
        let (merged, conflict) = three_way("a\nb\nc\n", "a\nmine\nc\n", "a\ntheirs\nc\n");
        assert!(conflict);
        assert!(merged.contains("<<<<<<< yours\nmine\n=======\ntheirs\n>>>>>>> rune-ui"));
    }

    #[test]
    fn a_unified_diff_shows_only_what_changed() {
        let d = unified("a\nb\nc\n", "a\nx\nc\n", "registry", "yours", 1);
        assert!(d.contains("-b\n+x\n"));
        assert!(unified("a\n", "a\n", "x", "y", 1).is_empty());
    }
}
