/**
 * Widget Demo - Showcasing all the new widgets in the TypeScript SDK
 * 
 * This example demonstrates how to use the various widgets that have been
 * added to the builder, including DataTable, Modal, Progress Bar, Tabs,
 * Toast notifications, and more.
 */

import {
    Terminal,
    Renderer,
    Component,
    DataTable,
    FilterType,
    TableColumn,
    TableRow,
    PaginationConfig,
    initialize,
    cleanup
} from 'reactive-tui';

// Sample data for the data table
const generateSampleData = (): TableRow[] => {
    const data: TableRow[] = [];
    const departments = ['Engineering', 'Sales', 'Marketing', 'HR', 'Finance'];
    const statuses = ['Active', 'On Leave', 'Remote', 'In Office'];
    
    for (let i = 1; i <= 100; i++) {
        data.push({
            id: i,
            data: {
                id: i,
                name: `Employee ${i}`,
                email: `employee${i}@company.com`,
                department: departments[Math.floor(Math.random() * departments.length)],
                salary: Math.floor(Math.random() * 50000) + 50000,
                hired: new Date(2020 + Math.floor(Math.random() * 5), 
                               Math.floor(Math.random() * 12), 
                               Math.floor(Math.random() * 28) + 1).toISOString().split('T')[0],
                status: statuses[Math.floor(Math.random() * statuses.length)],
                performance: Math.floor(Math.random() * 100)
            }
        });
    }
    
    return data;
};

/**
 * Dashboard Component with multiple widgets
 */
class DashboardComponent extends Component {
    private dataTable: DataTable;
    private currentFilter: string = '';
    private sortColumn: string = 'id';
    private sortDirection: 'asc' | 'desc' = 'asc';
    
    constructor() {
        super();
        
        // Initialize the data table
        this.dataTable = new DataTable({
            columns: [
                { key: 'id', title: 'ID', width: 8, sortable: true },
                { key: 'name', title: 'Name', width: 20, sortable: true, filterable: true },
                { key: 'email', title: 'Email', width: 25, sortable: true, filterable: true },
                { key: 'department', title: 'Department', width: 15, sortable: true, filterable: true },
                { key: 'salary', title: 'Salary', width: 12, sortable: true, 
                  formatter: (val) => `$${val.toLocaleString()}` },
                { key: 'hired', title: 'Hired Date', width: 12, sortable: true },
                { key: 'status', title: 'Status', width: 12, filterable: true },
                { key: 'performance', title: 'Performance', width: 12, sortable: true,
                  formatter: (val) => `${val}%` }
            ],
            rows: generateSampleData(),
            pagination: {
                currentPage: 0,
                pageSize: 20,
                totalRows: 100,
                showPageInfo: true,
                showNavigation: true
            },
            sortColumn: 'id',
            sortDirection: 'asc',
            selectable: true,
            virtualScroll: true,
            height: 30,
            class: 'employee-table'
        });
    }
    
    /**
     * Handle keyboard input for table interactions
     */
    handleInput(key: string): void {
        switch(key) {
            // Navigation
            case 'ArrowUp':
                this.dataTable.previousPage();
                break;
            case 'ArrowDown':
                this.dataTable.nextPage();
                break;
            
            // Sorting
            case 's':
                this.showSortMenu();
                break;
            
            // Filtering
            case 'f':
                this.showFilterDialog();
                break;
            
            // Clear filters
            case 'c':
                this.dataTable.clearFilters();
                console.log('Filters cleared');
                break;
            
            // Export
            case 'e':
                this.exportData();
                break;
            
            // Toggle column visibility
            case 'v':
                this.toggleColumnVisibility();
                break;
            
            // Refresh
            case 'r':
                this.dataTable.refresh();
                console.log('Table refreshed');
                break;
            
            // Help
            case 'h':
            case '?':
                this.showHelp();
                break;
            
            // Quit
            case 'q':
                this.cleanup();
                process.exit(0);
                break;
        }
    }
    
    /**
     * Show sort menu
     */
    private showSortMenu(): void {
        console.log('\n=== Sort Menu ===');
        console.log('1. Sort by ID');
        console.log('2. Sort by Name');
        console.log('3. Sort by Department');
        console.log('4. Sort by Salary');
        console.log('5. Sort by Performance');
        console.log('Press number to sort, or ESC to cancel');
        
        // In a real app, this would show a modal dialog
        // For demo, we'll just cycle through sort columns
        const columns = ['id', 'name', 'department', 'salary', 'performance'];
        const currentIndex = columns.indexOf(this.sortColumn);
        const nextIndex = (currentIndex + 1) % columns.length;
        this.sortColumn = columns[nextIndex];
        
        this.dataTable.toggleSort(this.sortColumn);
        console.log(`Sorted by ${this.sortColumn}`);
    }
    
    /**
     * Show filter dialog
     */
    private showFilterDialog(): void {
        console.log('\n=== Filter Options ===');
        console.log('1. Filter by Department');
        console.log('2. Filter by Status');
        console.log('3. Filter by Salary Range');
        console.log('4. Filter by Performance');
        console.log('5. Clear All Filters');
        
        // Demo: Add a sample filter
        this.dataTable.addFilter({
            columnKey: 'department',
            filterType: FilterType.Contains,
            value: 'Engineering',
            active: true
        });
        
        console.log('Applied filter: Department contains "Engineering"');
    }
    
    /**
     * Export data
     */
    private exportData(): void {
        console.log('\n=== Export Options ===');
        console.log('1. Export to CSV');
        console.log('2. Export to JSON');
        
        // Export to CSV
        const csv = this.dataTable.exportToCSV();
        console.log('\nCSV Export (first 500 chars):');
        console.log(csv.substring(0, 500) + '...');
        
        // Export to JSON
        const json = this.dataTable.exportToJSON();
        console.log('\nJSON Export (first 500 chars):');
        console.log(json.substring(0, 500) + '...');
    }
    
    /**
     * Toggle column visibility
     */
    private toggleColumnVisibility(): void {
        // Demo: Toggle email column visibility
        this.dataTable.toggleColumnVisibility('email');
        console.log('Toggled email column visibility');
    }
    
    /**
     * Show help menu
     */
    private showHelp(): void {
        console.log('\n=== DataTable Widget Demo - Help ===');
        console.log('Navigation:');
        console.log('  ↑/↓     - Previous/Next page');
        console.log('  s       - Sort menu');
        console.log('  f       - Filter menu');
        console.log('  c       - Clear all filters');
        console.log('  e       - Export data');
        console.log('  v       - Toggle column visibility');
        console.log('  r       - Refresh table');
        console.log('  h/?     - Show this help');
        console.log('  q       - Quit');
        console.log('\nCurrent State:');
        console.log(`  Sort: ${this.sortColumn} (${this.dataTable.getConfig().sortDirection})`);
        console.log(`  Filters: ${this.dataTable.getConfig().filters?.length || 0} active`);
        console.log(`  Page: ${this.dataTable.getConfig().pagination?.currentPage || 0 + 1}`);
    }
    
    render(): void {
        // In a real implementation, this would render the table
        // For now, we'll just log the current state
        const config = this.dataTable.getConfig();
        console.log('\n=== Employee DataTable ===');
        console.log(`Showing page ${(config.pagination?.currentPage || 0) + 1} of ${
            Math.ceil((config.pagination?.totalRows || 0) / (config.pagination?.pageSize || 20))
        }`);
        console.log(`Sorted by: ${config.sortColumn} (${config.sortDirection})`);
        console.log(`Active filters: ${config.filters?.filter(f => f.active).length || 0}`);
        console.log(`Selected rows: ${this.dataTable.getSelectedRows().length}`);
    }
}

/**
 * Main application demonstrating various widgets
 */
async function main() {
    console.log('=== Reactive TUI Widget Demo ===\n');
    
    try {
        // Initialize the library
        initialize();
        
        // Create terminal and renderer
        const terminal = new Terminal();
        const renderer = new Renderer(terminal);
        
        // Create dashboard component
        const dashboard = new DashboardComponent();
        
        // Show initial help
        console.log('Welcome to the Widget Demo!');
        console.log('This demo showcases the DataTable widget with advanced features:');
        console.log('- Filtering and sorting');
        console.log('- Pagination');
        console.log('- Column visibility control');
        console.log('- Data export (CSV/JSON)');
        console.log('- Virtual scrolling for performance\n');
        console.log('Press "h" for help, "q" to quit\n');
        
        // Initial render
        dashboard.render();
        
        // Set up input handling
        process.stdin.setRawMode(true);
        process.stdin.on('data', (data) => {
            const key = data.toString();
            dashboard.handleInput(key);
            dashboard.render();
        });
        
        // Demo: Show some widget interactions
        setTimeout(() => {
            console.log('\n--- Demo: Adding a filter ---');
            dashboard.handleInput('f');
        }, 2000);
        
        setTimeout(() => {
            console.log('\n--- Demo: Sorting data ---');
            dashboard.handleInput('s');
        }, 4000);
        
        setTimeout(() => {
            console.log('\n--- Demo: Navigating pages ---');
            dashboard.handleInput('ArrowDown');
        }, 6000);
        
    } catch (error) {
        console.error('Error:', error);
        cleanup();
        process.exit(1);
    }
}

// Run the demo
if (require.main === module) {
    main().catch(console.error);
}

export { DashboardComponent };