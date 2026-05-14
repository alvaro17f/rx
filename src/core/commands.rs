pub fn git_pull(repo: &str) -> String {
    format!("git -C {repo} pull")
}

pub fn git_diff(repo: &str) -> String {
    format!("git -C {repo} diff --exit-code")
}

pub fn git_status(repo: &str) -> String {
    format!("git -C {repo} status --porcelain")
}

pub fn git_add(repo: &str) -> String {
    format!("git -C {repo} add .")
}

pub fn nix_update(repo: &str) -> String {
    format!("nix flake update --flake {repo}")
}

pub fn nix_rebuild(repo: &str, hostname: &str) -> String {
    format!("sudo nixos-rebuild switch --flake {repo}#{hostname} --show-trace")
}

pub fn nix_keep(generations: u8) -> String {
    format!(
        "sudo nix-env --profile /nix/var/nix/profiles/system --delete-generations +{generations}"
    )
}

pub fn nix_diff() -> &'static str {
    "nix profile diff-closures --profile /nix/var/nix/profiles/system | tac | awk '/Version/{print; exit} 1' | tac"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_strings() {
        assert_eq!(git_pull("/repo"), "git -C /repo pull");
        assert_eq!(git_diff("/repo"), "git -C /repo diff --exit-code");
        assert_eq!(git_status("/repo"), "git -C /repo status --porcelain");
        assert_eq!(git_add("/repo"), "git -C /repo add .");
        assert_eq!(nix_update("/repo"), "nix flake update --flake /repo");
        assert_eq!(
            nix_rebuild("/repo", "host"),
            "sudo nixos-rebuild switch --flake /repo#host --show-trace"
        );
        assert_eq!(
            nix_keep(5),
            "sudo nix-env --profile /nix/var/nix/profiles/system --delete-generations +5"
        );
        assert!(nix_diff().contains("diff-closures"));
    }
}