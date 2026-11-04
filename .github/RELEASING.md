# Releasing

Adapted from the [release template](../docs/template/RELEASING.md.template) for thingblock-desktop's tag-triggered, GitHub Actions-built release flow.

## Channels

| Channel | Version / tag | Editor and link branch | GitHub release |
|---|---|---|---|
| Preview | `X.Y.Z-preview-YYYYMMDD` / `vX.Y.Z-preview-YYYYMMDD` | `develop` | pre-release |
| Official | `X.Y.Z` / `vX.Y.Z` | editor `main`, link `master` | latest release |

`develop` is the fast preview channel and may hold PRs that are not reviewed yet. `main` (editor) and `master` (link) hold reviewed, production-ready work only, so never merge an unreviewed PR into them.

To promote `develop` for an official release, in both the editor and the link repo:

1. Review what is new: `git log main..develop` and `git diff main...develop` (`master` in the link repo).
2. Revert on `develop` any PR that is not ready; a fast-forward takes everything.
3. Fast-forward and push: `git checkout main && git merge --ff-only origin/develop && git push origin main`.

## 1. Update the version

Bump the version number in all three places — they must match:

- `package.json` (`version`)
- `src-tauri/tauri.conf.json` (`version`)
- `src-tauri/Cargo.toml` (`[package].version`)

Also sync `package-lock.json` and `src-tauri/Cargo.lock`. The sibling repos carry their own versions — `thingblock-link`'s `Cargo.toml` and `thingblock-editor`'s `packages/thingblock-resource/package.json` — bump and push those to the branch the release builds from (see step 6) first.

## 2. Update `CHANGELOG.md`

For an official release, add a `## VERSION — DATE` section to `CHANGELOG.md` covering all three repos since the previous official tag, with the technical detail developers need. [GitHub automatically generated release notes] can serve as a starting point. Previews skip this step.

## 3. Commit changes

```sh
git commit -am "chore: release VERSION"
```

There shouldn't be other code changes in this commit.

## 4. Tag the release

```sh
git tag -a vVERSION -m "ThingBlock vVERSION"
```

## 5. Push changes

```sh
git push origin master
git push origin vVERSION
```

## 6. Build

Pushing a `v*` tag triggers `.github/workflows/release.yml`, which builds installers on three runners (macOS arm64, Windows x64, Linux x64) and uses `tauri-action` to draft a GitHub Release with four installer assets attached: `.exe`, `.dmg`, `.deb` and `.rpm`.

The workflow picks the editor and link branches from the tag: a tag containing `-preview-` checks out both at `develop` and marks the release as a pre-release; any other tag checks out editor `main` and link `master`. It builds from each branch's tip rather than the tag, so a change not yet on those branches does not ship, and a push to them during the run changes what ships. A manual `workflow_dispatch` run builds the official branches and uploads the installers as workflow artifacts without drafting a release.

## 7. Publish the GitHub release

Once all three matrix legs finish and the draft has exactly 4 assets, write the release notes, review, and publish it from the [releases page]. Release notes are for end users, while `CHANGELOG.md` is for developers: keep them short — what's new, what's fixed, what's removed, known limits, downloads — one line per item with no internals, and link to `CHANGELOG.md` for details. Preview notes open with a banner linking the stable release:

```md
> **Preview build** for testing. The stable release is [vX.Y.Z](https://github.com/ThingEdu/thingblock-desktop/releases/tag/vX.Y.Z).
```

```sh
gh release edit vVERSION --notes-file NOTES.md
```

The notes can also go in before the build: create the draft right after pushing the tag and `tauri-action` uploads the installers into it.

```sh
gh release create vVERSION --draft --title "ThingBlock vVERSION" --notes-file NOTES.md   # add --prerelease for a preview
```

GitHub renders every newline in a release body as a line break, so keep each paragraph and list item on one line in `NOTES.md`.

## 8. Announce the new release

Make sure to say "thank you" to the contributors who helped shape this version!

### Retriggering a release

If a build needs to be rerun against the same version:

```sh
git tag -f -a vVERSION -m "ThingBlock vVERSION"
git push origin :refs/tags/vVERSION
git push origin vVERSION
```

- Deleting a **draft** release in the GitHub UI also deletes its tag — re-push the tag afterward.
- Re-pushing a tag while a previous run is still in progress starts a duplicate run; cancel the old one first (`gh run cancel`).

[GitHub automatically generated release notes]: https://docs.github.com/en/repositories/releasing-projects-on-github/automatically-generated-release-notes#about-automatically-generated-release-notes
[releases page]: https://github.com/ThingEdu/thingblock-desktop/releases
