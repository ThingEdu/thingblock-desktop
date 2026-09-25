# Releasing

Adapted from [thoughtbot's release template](template/RELEASING.md.template) for thingblock-desktop's tag-triggered, GitHub Actions-built release flow.

## 1. Update the version

Bump the version number in all three places — they must match:

- `package.json` (`version`)
- `src-tauri/tauri.conf.json` (`version`)
- `src-tauri/Cargo.toml` (`[package].version`)

Also sync `package-lock.json` and `src-tauri/Cargo.lock`. The sibling repos carry their own versions — `thingblock-link`'s `Cargo.toml` and `thingblock-editor`'s `packages/thingblock-resource/package.json` — bump and push those to their release branches (see step 6) first.

## 2. Update `CHANGELOG.md`

Add a `## VERSION — DATE` section to `CHANGELOG.md` covering all three repos since the previous tag. [GitHub automatically generated release notes] can serve as a starting point.

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

The workflow checks out `thingblock-editor` at `main` and `thingblock-link` at `master` (pinned by `ref:`), building from each branch's tip rather than the tag — a change not yet on those branches does not ship. Do not drop the editor `ref`: its GitHub default branch `develop` is stale.

## 7. Publish the GitHub release

Once all three matrix legs finish and the draft has exactly 4 assets, set the draft's notes to the new `CHANGELOG.md` section, review, and publish it from the [releases page]:

```sh
gh release edit vVERSION --notes-file NOTES.md
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
