// Source: https://github.com/riyaz-ali/sqlite/blob/master/_examples/series/series.go
package main

import (
	"go.riyazali.net/sqlite"
)

//noinspection GoSnakeCaseUsage
const (
	SERIES_COLUMN_VALUE = iota
	SERIES_COLUMN_START
	SERIES_COLUMN_STOP
	SERIES_COLUMN_STEP
)

// SeriesModule implements a virtual table module that provides gen_series tables-valued function
// The source of this file is adapted from the c implementation at https://sqlite.org/src/file/ext/misc/series.c
type SeriesModule struct{}

func (s *SeriesModule) Connect(_ *sqlite.Conn, _ []string, declare func(string) error) (sqlite.VirtualTable, error) {
	return &SeriesTable{}, declare("CREATE TABLE series(value,start hidden,stop hidden,step hidden)")
}

type SeriesTable struct{}

func (s *SeriesTable) BestIndex(input *sqlite.IndexInfoInput) (*sqlite.IndexInfoOutput, error) {
	var idxNum = 0               // The query plan bitmask
	var unusableMask = 0         // Mask of unusable constraints
	var args = 0                 // Number of arguments that seriesFilter() expects
	var idx = [3]int{-1, -1, -1} // Constraints on start, stop, and step
	var output = &sqlite.IndexInfoOutput{
		ConstraintUsage: make([]*sqlite.ConstraintUsage, len(input.Constraints)),
	}

	for i, con := range input.Constraints {
		if con.ColumnIndex < SERIES_COLUMN_START {
			continue
		}
		column := con.ColumnIndex - SERIES_COLUMN_START
		mask := 1 << column
		if !con.Usable {
			unusableMask |= mask
			continue
		} else {
			idxNum |= mask
			idx[column] = i
		}
	}

	for i := 0; i < 3; i++ {
		j := idx[i]
		if j >= 0 {
			args += 1
			output.ConstraintUsage[j] = &sqlite.ConstraintUsage{ArgvIndex: args, Omit: false}
		}
	}

	if unusableMask&(^idxNum) != 0 {
		// The start, stop, and step columns are inputs.  Therefore if there
		// are unusable constraints on any of start, stop, or step then
		// this plan is unusable
		return nil, sqlite.SQLITE_CONSTRAINT
	}

	if (idxNum & 3) == 3 {
		// Both start= and stop= boundaries are available.  This is the the preferred case
		output.EstimatedCost = 2
		if (idxNum & 4) != 0 {
			output.EstimatedCost = 1
		}
		output.EstimatedRows = 1000
		if len(input.OrderBy) == 1 {
			if input.OrderBy[0].Desc {
				idxNum |= 8
			}
			output.OrderByConsumed = true
		}
	} else {
		// If either boundary is missing, we have to generate a huge span
		// of numbers.  Make this case very expensive so that the query
		// planner will work hard to avoid it.
		output.EstimatedRows = 2147483647
	}
	output.IndexNumber = idxNum
	return output, nil
}

func (s *SeriesTable) Open() (sqlite.VirtualCursor, error) { return &SeriesCursor{}, nil }
func (s *SeriesTable) Disconnect() error                   { return s.Destroy() }
func (s *SeriesTable) Destroy() error                      { return nil }

type SeriesCursor struct {
	desc     bool  // True to count down rather than up
	rowid    int64 // The rowid
	value    int64 // Current value ("value")
	minValue int64 // Minimum value ("start")
	maxValue int64 // Maximum value ("stop")
	step     int64 // Increment ("step")
}

func (cur *SeriesCursor) Filter(idxNum int, _ string, values ...sqlite.Value) error {
	if (idxNum & 1) != 0 {
		cur.minValue = values[0].Int64()
	} else {
		cur.minValue = 0
	}

	if (idxNum & 2) != 0 {
		cur.maxValue = values[1].Int64()
	} else {
		cur.maxValue = 0xffffffff
	}

	if (idxNum & 4) != 0 {
		cur.step = values[2].Int64()
		if cur.step < 1 {
			cur.step = 1
		}
	} else {
		cur.step = 1
	}

	for _, val := range values { // if any of the constraints have a NULL value, then return no rows.
		if val.Type() == sqlite.SQLITE_NULL {
			cur.minValue = 1
			cur.maxValue = 0
			break
		}
	}

	if (idxNum & 8) != 0 {
		cur.desc = true
		cur.value = cur.maxValue
		if cur.step > 0 {
			cur.value -= (cur.maxValue - cur.minValue) % cur.step
		}
	} else {
		cur.value = cur.minValue
	}
	cur.rowid = 1
	return nil
}

func (cur *SeriesCursor) Next() error {
	if cur.desc {
		cur.value -= cur.step
	} else {
		cur.value += cur.step
	}
	cur.rowid++
	return nil
}

func (cur *SeriesCursor) Column(context *sqlite.VirtualTableContext, i int) error {
	var x int64
	switch i {
	case SERIES_COLUMN_START:
		x = cur.minValue
	case SERIES_COLUMN_STOP:
		x = cur.maxValue
	case SERIES_COLUMN_STEP:
		x = cur.step
	default:
		x = cur.value
	}
	context.ResultInt64(x)
	return nil
}

func (cur *SeriesCursor) Eof() bool {
	if cur.desc {
		return cur.value < cur.minValue
	}
	return cur.value > cur.maxValue
}

func (cur *SeriesCursor) Rowid() (int64, error) { return cur.rowid, nil }
func (cur *SeriesCursor) Close() error          { return nil }

func init() {
	sqlite.Register(func(api *sqlite.ExtensionApi) (sqlite.ErrorCode, error) {
		if err := api.CreateModule("generate_series_go", &SeriesModule{},
			sqlite.EponymousOnly(true)); err != nil {
			return sqlite.SQLITE_ERROR, err
		}
		return sqlite.SQLITE_OK, nil
	})
}

func main() {}