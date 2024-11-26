// A generated module for Zero2Prod functions
//
// This module has been generated via dagger init and serves as a reference to
// basic module structure as you get started with Dagger.
//
// Two functions have been pre-created. You can modify, delete, or add to them,
// as needed. They demonstrate usage of arguments and return types using simple
// echo and grep commands. The functions can be called from the dagger CLI or
// from one of the SDKs.
//
// The first line in this comment block is a short description line and the
// rest is a long description with more detail on the module's purpose or usage,
// if appropriate. All modules should have a short description.

package main

import (
	"context"
	"dagger/zero-2-prod/internal/dagger"
	"fmt"
)

func New(
	ctx context.Context,
	// +optional
	// +defaultPath="/"
	// +ignore=["bin", ".git", "**/node_modules", "**/.venv", "**/__pycache__"]
	workspace *dagger.Directory,

	// +optional
	base *dagger.Container,
) (*Zero2Prod, error) {
	if base == nil {
		tag, err := dag.Github().GetLatestRelease("rust-lang/rust").Tag(ctx)
		if err != nil {
			return nil, err
		}
		base, err = dag.Container().
			From(fmt.Sprintf("rust:%s-alpine3.20", tag)).
			WithExec([]string{
				"apk",
				"add",
				"--no-cache",
				"cmake",
				"build-base",
				"libressl-dev",
				"musl-dev",
				"perl",
				"pkgconfig",
			}).
			WithExec([]string{
				"cargo", "install",
				"cargo-tarpaulin",
				"cargo-audit",
			}).
			WithExec([]string{
				"rustup", "component", "add",
				"clippy",
				"rustfmt",
			}).
			Sync(ctx)
		if err != nil {
			return nil, err
		}
	}
	base = base.WithUser("root").
		WithoutEnvVariable("CARGO_HOME").
		WithDirectory("/src", workspace).
		WithWorkdir("/src").
		WithoutEntrypoint()
	base = mountCaches(ctx, base)
	return &Zero2Prod{
		Workspace: workspace,
		Base:      base,
	}, nil
}

func mountCaches(ctx context.Context, base *dagger.Container) *dagger.Container {
	cargoRegistry := dag.CacheVolume("cargo_registry")
	cargoGit := dag.CacheVolume("cargo_git")

	return base.
		WithMountedCache("/root/.cargo/registry", cargoRegistry).
		WithMountedCache("/root/.cargo/git", cargoGit)
}

type Zero2Prod struct {
	Workspace *dagger.Directory
	Base      *dagger.Container
}

func (m *Zero2Prod) Test(ctx context.Context) (string, error) {
	return m.Base.WithExec([]string{"cargo", "test"}).Stderr(ctx)
}

func (m *Zero2Prod) Clippy(ctx context.Context) (string, error) {
	return m.Base.WithExec([]string{"cargo", "clippy", "--", "-D", "warnings"}).Stderr(ctx)
}

func (m *Zero2Prod) FormatCheck(ctx context.Context) (string, error) {
	return m.Base.WithExec([]string{"cargo", "fmt", "--", "--check"}).Stderr(ctx)
}

func (m *Zero2Prod) Audit(ctx context.Context) (string, error) {
	return m.Base.WithExec([]string{"cargo", "audit"}).Stderr(ctx)
}
