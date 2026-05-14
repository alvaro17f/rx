/// Build a `git pull` command for `repo`.
pub fn git_pull(repo: &str) -> String {
    format!("git -C {repo} pull")
}

/// Build a `git diff --exit-code` command for `repo`.
pub fn git_diff(repo: &str) -> String {
    format!("git -C {repo} diff --exit-code")
}

/// Build a `git status --porcelain` command for `repo`.
pub fn git_status(repo: &str) -> String {
    format!("git -C {repo} status --porcelain")
}

/// Build a `git add .` command for `repo`.
pub fn git_add(repo: &str) -> String {
    format!("git -C {repo} add .")
}

/// Build a `nix flake update` command for `repo`.
pub fn nix_update(repo: &str) -> String {
    format!("nix flake update --flake {repo}")
}

/// Build a `nixos-rebuild switch` command for `repo` and `hostname`.
pub fn nix_rebuild(repo: &str, hostname: &str) -> String {
    format!("sudo nixos-rebuild switch --flake {repo}#{hostname} --show-trace")
}

/// Build a `nix-env --delete-generations` command keeping `generations`.
pub fn nix_keep(generations: u8) -> String {
    format!(
        "sudo nix-env --profile /nix/var/nix/profiles/system --delete-generations +{generations}"
    )
}

/// Return the `nix profile diff-closures` pipeline string.
pub fn nix_diff() -> &'static str {
    "nix profile diff-closures --profile /nix/var/nix/profiles/system | tac | awk '/Version/{print; exit} 1' | tac"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_pull_format() {
        assert_eq!(git_pull("/repo"), "git -C /repo pull");
    }

    #[test]
    fn git_diff_format() {
        assert_eq!(git_diff("/repo"), "git -C /repo diff --exit-code");
    }

    #[test]
    fn git_status_format() {
        assert_eq!(git_status("/repo"), "git -C /repo status --porcelain");
    }

    #[test]
    fn git_add_format() {
        assert_eq!(git_add("/repo"), "git -C /repo add .");
    }

    #[test]
    fn nix_update_format() {
        assert_eq!(nix_update("/repo"), "nix flake update --flake /repo");
    }

    #[test]
    fn nix_rebuild_format() {
        assert_eq!(
            nix_rebuild("/repo", "host"),
            "sudo nixos-rebuild switch --flake /repo#host --show-trace"
        );
    }

    #[test]
    fn nix_keep_format() {
        assert_eq!(
            nix_keep(5),
            "sudo nix-env --profile /nix/var/nix/profiles/system --delete-generations +5"
        );
    }

    #[test]
    fn nix_diff_contains_diff_closures() {
        assert!(nix_diff().contains("diff-closures"));
    }
}