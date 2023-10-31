package main

import (
	"context"
	"fmt"
	"github.com/docker/docker/api/types"
	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/client"
)

// 1. fetch env variables for postgres
// 2. deploy postgres image with env vars on docker
// 3. setup postgres data with db_prep_tool
// 4. setup env vars for accounting system
// 5. dockerize accounting system and deploy it to docker with postgres details
// 6. start user journey tests (should be done in 10-30s)
func main() {
	ctx := context.Background()
	var env []string
	for key, value := range map[string]string{"POSTGRES_USER": "postgres", "POSTGRES_PASSWORD": "postgres", "POSTGRES_DB": "postgres"} {
		env = append(env, fmt.Sprintf("%s=%s", key, value))
	}
	cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
	if err != nil {
		panic(err)
	}
	defer func(cli *client.Client) {
		err := cli.Close()
		if err != nil {
			panic(err)
		}
	}(cli)
	//cli.
	//reader, err := cli.ImagePull(ctx, "docker.io/library/postgres:16.0-alpine3.18", types.ImagePullOptions{})
	//if err != nil {
	//	panic(err)
	//}
	//io.Copy(os.Stdout, reader)
	cr, errcc := cli.ContainerCreate(ctx, &container.Config{
		Image: "postgres:16.0-alpine3.18",
		Env:   env,
	}, nil, nil, nil, "my-postgres-test")
	if errcc != nil {
		panic(err)
	}
	println(" created container id %s", cr.ID)
	errs := cli.ContainerStart(ctx, cr.ID, types.ContainerStartOptions{})
	if errs != nil {
		panic(err)
	}
	ci, erri := cli.ContainerInspect(ctx, cr.ID)
	if erri != nil {
		panic(err)
	}
	p := ci.NetworkSettings.IPAddress
	println("ip address ", p)
	var env2 []string
	for key, value := range map[string]string{
		"POSTGRES_PORT":                    "5432",
		"POSTGRES_USER":                    "postgres",
		"POSTGRES_DB":                      "postgres",
		"POSTGRES_MAX_CONNECTIONS":         "5",
		"POSTGRES_HOST":                    p,
		"POSTGRES_connect_timeout_seconds": "10",
		"POSTGRES_PASSWORD":                "postgres",
		"POSTGRES_WAIT_TIMEOUT_SECONDS":    "5",
		"POSTGRES_POOL_RECYCLING_METHOD":   "Clean",
		"POSTGRES_APPLICATION_NAME":        "accounting-system",
	} {
		env2 = append(env2, fmt.Sprintf("%s=%s", key, value))
	}
	ccc, errccc := cli.ContainerCreate(ctx, &container.Config{
		Image: "accounting-system-now:latest",
		Env:   env2,
	}, nil, nil, nil, "accounting_system")
	if errccc != nil {
		panic(errccc)
	}
	println("", ccc.ID)
	//cli.
	println("ljlkjfdalkjdfa")
}
