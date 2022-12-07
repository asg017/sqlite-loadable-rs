package main

import (
	"fmt"

	"go.riyazali.net/sqlite"
)

type Yo struct{}

func (m *Yo) Args() int           { return 0 }
func (m *Yo) Deterministic() bool { return true }
func (m *Yo) Apply(ctx *sqlite.Context, values ...sqlite.Value) {
	ctx.ResultText("yo")
}


type Surround struct{}

func (m *Surround) Args() int           { return 1 }
func (m *Surround) Deterministic() bool { return true }
func (m *Surround) Apply(ctx *sqlite.Context, values ...sqlite.Value) {
	ctx.ResultText(fmt.Sprintf("x%sx", values[0].Text()))
}

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
		if err := api.CreateFunction("yo_go", &Yo{}); err != nil {
			return sqlite.SQLITE_ERROR, err
		}
		if err := api.CreateFunction("surround_go", &Surround{}); err != nil {
			return sqlite.SQLITE_ERROR, err
		}
		return sqlite.SQLITE_OK, nil
	})
}

func main() {}