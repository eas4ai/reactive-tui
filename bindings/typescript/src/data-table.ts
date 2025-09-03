/**
 * Data Table Widget - Advanced table with filtering, sorting, and pagination
 */

import { FFI } from './ffi';
import { ReactiveError } from './error';
import { Component } from './component';
import { ElementHandle, ComponentHandle } from './types';

/**
 * Filter types for table columns
 */
export enum FilterType {
    Contains = 'contains',
    Equals = 'equals',
    Range = 'range',
    DateRange = 'date_range',
    Boolean = 'boolean'
}

/**
 * Column filter configuration
 */
export interface ColumnFilter {
    columnKey: string;
    filterType: FilterType;
    value: string | number | boolean | [number, number] | [string, string];
    active: boolean;
}

/**
 * Pagination configuration
 */
export interface PaginationConfig {
    currentPage: number;
    pageSize: number;
    totalRows: number;
    showPageInfo?: boolean;
    showNavigation?: boolean;
}

/**
 * Table column definition
 */
export interface TableColumn {
    key: string;
    title: string;
    width?: number;
    sortable?: boolean;
    filterable?: boolean;
    visible?: boolean;
    align?: 'left' | 'center' | 'right';
    formatter?: (value: any) => string;
}

/**
 * Table row data
 */
export interface TableRow {
    id: string | number;
    data: Record<string, any>;
    selected?: boolean;
    expanded?: boolean;
}

/**
 * Data table configuration
 */
export interface DataTableConfig {
    columns: TableColumn[];
    rows: TableRow[];
    filters?: ColumnFilter[];
    pagination?: PaginationConfig;
    sortColumn?: string;
    sortDirection?: 'asc' | 'desc';
    multiSort?: boolean;
    selectable?: boolean;
    expandable?: boolean;
    virtualScroll?: boolean;
    height?: number;
    width?: number;
    class?: string;
}

/**
 * Data Table Widget
 * 
 * Advanced table component with filtering, sorting, pagination,
 * and virtual scrolling capabilities.
 * 
 * @example
 * ```typescript
 * const table = new DataTable({
 *     columns: [
 *         { key: 'id', title: 'ID', width: 10, sortable: true },
 *         { key: 'name', title: 'Name', sortable: true, filterable: true },
 *         { key: 'age', title: 'Age', sortable: true, filterable: true }
 *     ],
 *     rows: [
 *         { id: 1, data: { id: 1, name: 'John', age: 30 } },
 *         { id: 2, data: { id: 2, name: 'Jane', age: 25 } }
 *     ],
 *     pagination: {
 *         currentPage: 0,
 *         pageSize: 10,
 *         totalRows: 100
 *     }
 * });
 * ```
 */
export class DataTable extends Component {
    private config: DataTableConfig;
    private tableHandle?: ComponentHandle;

    constructor(config: DataTableConfig) {
        // Create a component for the data table
        const handle = FFI.componentCreate('DataTable');
        if (!handle) {
            throw new ReactiveError('Failed to create DataTable component');
        }
        super(handle);
        
        this.config = config;
        this.initialize();
    }

    private initialize(): void {
        // Initialize table with configuration
        this.updateColumns(this.config.columns);
        this.updateRows(this.config.rows);
        
        if (this.config.filters) {
            this.setFilters(this.config.filters);
        }
        
        if (this.config.pagination) {
            this.setPagination(this.config.pagination);
        }
        
        if (this.config.sortColumn) {
            this.setSorting(this.config.sortColumn, this.config.sortDirection || 'asc');
        }
    }

    /**
     * Update table columns
     */
    updateColumns(columns: TableColumn[]): void {
        // Convert columns to FFI format and update via FFI
        const columnData = JSON.stringify(columns);
        
        // Call FFI function to update component state
        const result = FFI.lib.rtui_component_set_state(
            this.handle, 
            JSON.stringify({ columns })
        );
        
        if (result !== 0) {
            throw new ReactiveError(
                `Failed to update table columns: error code ${result}`,
                'DataTableError'
            );
        }
        
        // Update local state cache
        this.setState({ columns });
    }

    /**
     * Update table rows
     */
    updateRows(rows: TableRow[]): void {
        this.setState({ rows });
    }

    /**
     * Apply filters to the table
     */
    setFilters(filters: ColumnFilter[]): void {
        this.config.filters = filters;
        this.applyFilters();
    }

    /**
     * Add a new filter
     */
    addFilter(filter: ColumnFilter): void {
        if (!this.config.filters) {
            this.config.filters = [];
        }
        this.config.filters.push(filter);
        this.applyFilters();
    }

    /**
     * Remove a filter by column key
     */
    removeFilter(columnKey: string): void {
        if (!this.config.filters) return;
        
        this.config.filters = this.config.filters.filter(
            f => f.columnKey !== columnKey
        );
        this.applyFilters();
    }

    /**
     * Clear all filters
     */
    clearFilters(): void {
        this.config.filters = [];
        this.applyFilters();
    }

    private applyFilters(): void {
        // Apply filters to rows
        let filteredRows = [...this.config.rows];
        
        if (this.config.filters) {
            for (const filter of this.config.filters) {
                if (!filter.active) continue;
                
                filteredRows = filteredRows.filter(row => {
                    const value = row.data[filter.columnKey];
                    
                    switch (filter.filterType) {
                        case FilterType.Contains:
                            return String(value).toLowerCase().includes(
                                String(filter.value).toLowerCase()
                            );
                        case FilterType.Equals:
                            return value === filter.value;
                        case FilterType.Range:
                            const [min, max] = filter.value as [number, number];
                            return value >= min && value <= max;
                        case FilterType.Boolean:
                            return Boolean(value) === filter.value;
                        default:
                            return true;
                    }
                });
            }
        }
        
        this.setState({ filteredRows });
    }

    /**
     * Set pagination configuration
     */
    setPagination(config: PaginationConfig): void {
        this.config.pagination = config;
        this.setState({ pagination: config });
    }

    /**
     * Navigate to a specific page
     */
    goToPage(page: number): void {
        if (!this.config.pagination) return;
        
        const maxPage = Math.ceil(
            this.config.pagination.totalRows / this.config.pagination.pageSize
        ) - 1;
        
        this.config.pagination.currentPage = Math.max(0, Math.min(page, maxPage));
        this.setState({ pagination: this.config.pagination });
    }

    /**
     * Go to next page
     */
    nextPage(): void {
        if (!this.config.pagination) return;
        this.goToPage(this.config.pagination.currentPage + 1);
    }

    /**
     * Go to previous page
     */
    previousPage(): void {
        if (!this.config.pagination) return;
        this.goToPage(this.config.pagination.currentPage - 1);
    }

    /**
     * Set sorting configuration
     */
    setSorting(column: string, direction: 'asc' | 'desc' = 'asc'): void {
        this.config.sortColumn = column;
        this.config.sortDirection = direction;
        this.applySorting();
    }

    /**
     * Toggle sort direction for a column
     */
    toggleSort(column: string): void {
        if (this.config.sortColumn === column) {
            this.config.sortDirection = 
                this.config.sortDirection === 'asc' ? 'desc' : 'asc';
        } else {
            this.config.sortColumn = column;
            this.config.sortDirection = 'asc';
        }
        this.applySorting();
    }

    private applySorting(): void {
        if (!this.config.sortColumn) return;
        
        const sortedRows = [...this.config.rows].sort((a, b) => {
            const aVal = a.data[this.config.sortColumn!];
            const bVal = b.data[this.config.sortColumn!];
            
            let comparison = 0;
            if (aVal < bVal) comparison = -1;
            if (aVal > bVal) comparison = 1;
            
            return this.config.sortDirection === 'asc' ? comparison : -comparison;
        });
        
        this.setState({ sortedRows });
    }

    /**
     * Select a row
     */
    selectRow(rowId: string | number): void {
        const row = this.config.rows.find(r => r.id === rowId);
        if (row) {
            row.selected = true;
            this.setState({ rows: this.config.rows });
        }
    }

    /**
     * Deselect a row
     */
    deselectRow(rowId: string | number): void {
        const row = this.config.rows.find(r => r.id === rowId);
        if (row) {
            row.selected = false;
            this.setState({ rows: this.config.rows });
        }
    }

    /**
     * Get selected rows
     */
    getSelectedRows(): TableRow[] {
        return this.config.rows.filter(r => r.selected);
    }

    /**
     * Export table data to CSV
     */
    exportToCSV(): string {
        const headers = this.config.columns
            .filter(col => col.visible !== false)
            .map(col => col.title);
        
        const rows = this.config.rows.map(row => {
            return this.config.columns
                .filter(col => col.visible !== false)
                .map(col => {
                    const value = row.data[col.key];
                    return col.formatter ? col.formatter(value) : String(value);
                });
        });
        
        const csv = [
            headers.join(','),
            ...rows.map(row => row.join(','))
        ].join('\n');
        
        return csv;
    }

    /**
     * Export table data to JSON
     */
    exportToJSON(): string {
        const data = this.config.rows.map(row => {
            const obj: Record<string, any> = {};
            this.config.columns
                .filter(col => col.visible !== false)
                .forEach(col => {
                    obj[col.key] = row.data[col.key];
                });
            return obj;
        });
        
        return JSON.stringify(data, null, 2);
    }

    /**
     * Show/hide a column
     */
    toggleColumnVisibility(columnKey: string): void {
        const column = this.config.columns.find(c => c.key === columnKey);
        if (column) {
            column.visible = column.visible === false ? true : false;
            this.setState({ columns: this.config.columns });
        }
    }

    /**
     * Resize a column
     */
    resizeColumn(columnKey: string, width: number): void {
        const column = this.config.columns.find(c => c.key === columnKey);
        if (column) {
            column.width = width;
            this.setState({ columns: this.config.columns });
        }
    }

    /**
     * Enable virtual scrolling for large datasets
     */
    enableVirtualScroll(height: number): void {
        this.config.virtualScroll = true;
        this.config.height = height;
        this.setState({ virtualScroll: true, height });
    }

    /**
     * Refresh table data
     */
    refresh(): void {
        this.applyFilters();
        this.applySorting();
        this.render();
    }

    /**
     * Clear all table data
     */
    clear(): void {
        this.config.rows = [];
        this.setState({ rows: [] });
    }

    /**
     * Get current table configuration
     */
    getConfig(): DataTableConfig {
        return { ...this.config };
    }
}