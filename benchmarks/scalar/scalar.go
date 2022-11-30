package main

import (
	"go.riyazali.net/sqlite"
)

type Add struct{}

func (m *Add) Args() int           { return 2 }
func (m *Add) Deterministic() bool { return true }
func (m *Add) Apply(ctx *sqlite.Context, values ...sqlite.Value) {
	ctx.ResultInt(values[0].Int()+values[1].Int())
}

func init() {
	sqlite.Register(func(api *sqlite.ExtensionApi) (sqlite.ErrorCode, error) {
		if err := api.CreateFunction("add_go", &Add{}); err != nil {
			return sqlite.SQLITE_ERROR, err
		}
		return sqlite.SQLITE_OK, nil
	})
}

func main() {}