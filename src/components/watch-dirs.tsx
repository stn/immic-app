import { useState, useCallback } from "react";
import { dialog } from "@tauri-apps/api";

import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  RowSelectionState,
  useReactTable,
} from "@tanstack/react-table"

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";

export type WatchDir = {
    path: string
}
 
export const columns: ColumnDef<WatchDir>[] = [
    {
        id: "select",
        header: ({ table }) => (
            <Checkbox
                checked={table.getIsAllPageRowsSelected() || (table.getIsSomePageRowsSelected() && "indeterminate")}
                onCheckedChange={(value) => table.toggleAllPageRowsSelected(!!value)}
                aria-label="Select all"
            />
        ),
        cell: ({ row }) => (
            <Checkbox
                checked={row.getIsSelected()}
                onCheckedChange={(value) => row.toggleSelected(!!value)}
                aria-label="Select row"
            />
        ),
    },
    {
        accessorKey: "path",
        header: "Path",
    },
]

export interface WatchDirsProps {
  dirs: WatchDir[]
  setDirs: (dirs: WatchDir[]) => void
}

export function WatchDirTable({ dirs, setDirs }: WatchDirsProps) {
    const [rowSelection, setRowSelection] = useState<RowSelectionState>({})

    const table = useReactTable({
        data: dirs,
        columns,
        getCoreRowModel: getCoreRowModel(),
        onRowSelectionChange: setRowSelection,
        state: {
            rowSelection,
        },
    });

    const addWatchDir = useCallback(() => {
        (async () => {
            const selected = await dialog.open({
                directory: true,
                multiple: true,
            });
            let newDirs: WatchDir[] = [];
            if (selected) {
                if (Array.isArray(selected)) {
                selected.forEach((path) => {
                    newDirs.push({ path: path });
                });
                } else if (typeof selected === "string" && selected !== "") {
                newDirs.push({ path: selected });
                }
                // TODO: escape '|' in path
                setDirs([...dirs, ...newDirs]);
            }
        })();
    }, [dirs]);

    const removeWatchDir = useCallback(() => {
        let newDirs: WatchDir[] = [];
        for (let i = 0; i < dirs.length; i++) {
            if (!rowSelection[i]) {
                newDirs.push(dirs[i]);
            }
        }
        if (newDirs.length === dirs.length) {
            return;
        }
        setDirs(newDirs);
        setRowSelection({});
    }, [dirs, rowSelection]);

    return (
        <div className="flex flex-rows grid">
            <div className="rounded-md border">
                <Table>
                    <TableHeader>
                        {table.getHeaderGroups().map((headerGroup) => (
                            <TableRow key={headerGroup.id}>
                                {headerGroup.headers.map((header) => {
                                    return (
                                        <TableHead key={header.id}>
                                            {header.isPlaceholder
                                            ? null
                                            : flexRender(
                                                header.column.columnDef.header,
                                                header.getContext()
                                                )}
                                        </TableHead>
                                    );
                                })}
                            </TableRow>
                        ))}
                    </TableHeader>
                    <TableBody>
                        {table.getRowModel().rows?.length ? (
                            table.getRowModel().rows.map((row) => (
                                <TableRow
                                    key={row.id}
                                    data-state={row.getIsSelected() && "selected"}
                                >
                                    {row.getVisibleCells().map((cell) => (
                                    <TableCell key={cell.id}>
                                        {flexRender(cell.column.columnDef.cell, cell.getContext())}
                                    </TableCell>
                                    ))}
                                </TableRow>
                            ))
                        ) : (
                            <TableRow>
                                <TableCell colSpan={columns.length} className="h-24 text-center">
                                    {" "}
                                </TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </div>
            <div className="flex flex-cols gap-4">
                <Button
                    className="w-20"
                    variant="outline"
                    onClick={(e) => {
                        e.preventDefault();
                        addWatchDir();
                    }}
                >
                +
                </Button>
                <Button
                    className="w-20"
                    variant="outline"
                    onClick={(e) => {
                        e.preventDefault();
                        removeWatchDir();
                    }}
                >
                -
                </Button>
            </div>
        </div>
    );
}
