# Releasing

Adapted from [thoughtbot's release template](template/RELEASING.md.template) for
thingblock-desktop's tag-triggered, GitHub Actions-built release flow.

## 1. Update the version

Bump the version number in all three places — they must match:

- `package.json` (`version`)
- `src-tauri/tauri.conf.json` (`version`)
- `src-tauri/Cargo.toml` (`[package].version`)

## 2. Update release notes

There is no `NEWS.md`/`CHANGELOG.md` in this repo. Release notes come from GitHub's
[automatically generated release notes] (compare against the previous tag), used either
as-is by `tauri-action` or pasted into a pre-created draft (see step 6).

## 3. Commit changes

```sh
git commit -am "chore: bump version to vVERSION"
```

There shouldn't be other code changes in this commit.

## 4. Tag the release

```sh
git tag -a vVERSION -m "vVERSION"
```

## 5. Push changes

```sh
git push origin master
git push origin vVERSION
```

## 6. Build and publish

Pushing a `v*` tag triggers `.github/workflows/release.yml`, which builds installers on
four runners (macOS arm64, Windows x64, Linux x64, Linux arm64) and uses `tauri-action` to
draft a GitHub Release with the six installer assets (.exe, .dmg, x64+arm64 .deb/.rpm)
attached.

Optionally pre-create the draft release right after pushing the tag, so it carries real
release notes instead of `tauri-action`'s default:

```sh
gh release create vVERSION --draft --title "ThingBlock vVERSION" --notes-file NOTES.md
```

`tauri-action` reuses the existing draft for the tag and uploads assets into it.

## 7. Publish the GitHub release

Once all four matrix legs finish and the draft has exactly 6 assets, review and publish it
from the [releases page].

## 8. Announce the new release

Make sure to say "thank you" to the contributors who helped shape this version!

### Retriggering a release

If a build needs to be rerun against the same version:

```sh
git tag -f -a vVERSION -m "vVERSION"
git push origin :refs/tags/vVERSION
git push origin vVERSION
```

- Deleting a **draft** release in the GitHub UI also deletes its tag — re-push the tag
  afterward.
- Re-pushing a tag while a previous run is still in progress starts a duplicate run;
  cancel the old one first (`gh run cancel`).
- `release.yml` checks out `thingblock-desktop`, `thingblock-editor`, and `thingblock-link`
  by explicit `repository:` with no `ref`, so each builds from its default branch — a
  version bump stranded on a feature branch in a sibling repo silently ships the old code.

[automatically generated release notes]: https://docs.github.com/en/repositories/releasing-projects-on-github/automatically-generated-release-notes#about-automatically-generated-release-notes
[releases page]: https://github.com/ThingEdu/thingblock-desktop/releases
