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

	"github.com/google/uuid"
)

func New(
	ctx context.Context,
	// +optional
	// +defaultPath="/"
	// +ignore=["bin", ".git", "**/node_modules", "**/.venv", "**/__pycache__"]
	workspace *dagger.Directory,

	// +optional
	base *dagger.Container,

	// +optional
	// +default="1.82"
	tag string,
) (*Zero2Prod, error) {
	var err error
	if tag == "latest" {
		tag, err = dag.Github().GetLatestRelease("rust-lang/rust").Tag(ctx)
		if err != nil {
			return nil, err
		}
	}
	if base == nil {
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
				"sqlx-cli",
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
		WithWorkdir("/src")
	base = mountCaches(base)
	return &Zero2Prod{
		Workspace: workspace,
		Base:      base,
	}, nil
}

func mountCaches(base *dagger.Container) *dagger.Container {
	cargoCache := dag.CacheVolume("cargo")

	return base.
		WithMountedCache("/root/.cargo", cargoCache)
}

type Zero2Prod struct {
	Workspace *dagger.Directory
	Base      *dagger.Container
}

func WithDatabase(container *dagger.Container) *dagger.Container {
	return container.WithServiceBinding("database",
		dag.Container().From("postgres:17").
			WithEnvVariable("POSTGRES_USER", "postgres").
			WithEnvVariable("POSTGRES_DB", "newsletter").
			WithEnvVariable("POSTGRES_PASSWORD", "password").
			WithExposedPort(5432, dagger.ContainerWithExposedPortOpts{
				ExperimentalSkipHealthcheck: false,
			}).
			AsService(dagger.ContainerAsServiceOpts{
				UseEntrypoint: true,
			}),
	).
		WithEnvVariable("DATABASE_URL", "postgres://postgres:password@database:5432/newsletter").
		WithEnvVariable("NO_CACHE_MARK", uuid.New().String()).
		WithExec([]string{"cargo", "sqlx", "migrate", "run"})
}

func (m *Zero2Prod) Test(ctx context.Context) (string, error) {
	return WithDatabase(m.Base).
		WithExec([]string{"cargo", "test"}).Stderr(ctx)
}

func (m *Zero2Prod) Clippy(ctx context.Context) (string, error) {
	return WithDatabase(m.Base).
		WithExec([]string{"cargo", "clippy", "--", "-D", "warnings"}).Stderr(ctx)
}

func (m *Zero2Prod) FormatCheck(ctx context.Context) (string, error) {
	return m.Base.WithExec([]string{"cargo", "fmt", "--", "--check"}).Stderr(ctx)
}

func (m *Zero2Prod) Audit(ctx context.Context) (string, error) {
	return WithDatabase(m.Base).
		WithExec([]string{"cargo", "audit"}).Stderr(ctx)
}
