//! # rye-ui
//!
//! Pre-built UI component library for the rye framework.
//!
//! Provides a comprehensive set of reusable components organized into tiers:
//!
//! - **Tier 1 (Forms):** Button, Input, Textarea, Select, Checkbox, Radio, Switch, Label
//! - **Tier 2 (Layout):** Box, Flex, Grid, Stack, Card, Divider, Container
//! - **Tier 3 (Feedback):** Dialog, Toast, Tooltip, Popover, Alert, Spinner, Progress, Skeleton
//! - **Tier 4 (Navigation):** Tabs, Accordion, Breadcrumb, Link
//! - **Tier 5 (Data):** Table, Badge, Avatar, Tag, List
//! - **Tier 6 (Advanced Forms):** FormField, Slider, DatePicker, FileUpload
//! - **Tier 7 (Overlay & Interaction):** Dropdown, ContextMenu, CommandPalette, BottomSheet, Drawer, HoverCard
//! - **Tier 8 (Data Display — Advanced):** DataTable, TreeView, Timeline, Calendar, Carousel, CodeBlock, EmptyState, Stat
//! - **Tier 9 (Forms — Advanced):** ColorPicker, Rating, OtpInput, MaskedInput, TagInput, AutoComplete, FormValidator
//! - **Tier 10 (Layout — Advanced):** AspectRatio, Collapsible, Resizable, ScrollArea, VirtualList
//! - **Tier 11 (Feedback — Advanced):** ConfirmationDialog, CircularProgress, Notification, ErrorBoundary
//!
//! Each component is a pure function that returns an [`Element`], following
//! the rye component model. Components accept typed props structs for
//! compile-time safety.
//!
//! ## Theming
//!
//! All components use CSS custom properties (variables) for colors, spacing,
//! and typography — no hardcoded color values. Use [`ThemeProvider`] to inject
//! the variables into the DOM:
//!
//! ```ignore
//! use rye_ui::{ThemeProvider, ThemeProviderProps};
//!
//! // Light theme (default)
//! ThemeProvider::render(ThemeProviderProps::light());
//!
//! // Dark theme
//! ThemeProvider::render(ThemeProviderProps::dark());
//!
//! // Auto — follows system preference
//! ThemeProvider::render(ThemeProviderProps::auto());
//! ```
//!
//! Components reference `var(--rye-primary)`, `var(--rye-bg)`, etc. via the
//! [`vars`] module constants (e.g. [`vars::PRIMARY`], [`vars::BG`], [`vars::TEXT`]).
//! Switching themes at runtime only requires changing the `data-theme` attribute
//! — no re-render needed.
//!
//! See [`tokens`] for the full list of design tokens and CSS variable constants.

pub mod button;
pub mod checkbox;
pub mod input;
pub mod label;
pub mod radio;
pub mod select;
pub mod switch;
pub mod textarea;
pub mod theme;
pub mod theme_provider;
pub mod tokens;

pub mod box_;
pub mod card;
pub mod container;
pub mod divider;
pub mod flex;
pub mod grid;
pub mod stack;

pub mod alert;
pub mod dialog;
pub mod popover;
pub mod progress;
pub mod skeleton;
pub mod spinner;
pub mod toast;
pub mod tooltip;

pub mod accordion;
pub mod breadcrumb;
pub mod link;
pub mod tabs;

pub mod avatar;
pub mod badge;
pub mod list;
pub mod table;
pub mod tag;

pub mod date_picker;
pub mod file_upload;
pub mod form_field;
pub mod slider;

// Re-export all components at the crate root for convenience.
pub use button::{Button, ButtonProps};
pub use checkbox::{Checkbox, CheckboxProps};
pub use input::{Input, InputProps};
pub use label::{Label, LabelProps};
pub use radio::{Radio, RadioGroup, RadioGroupProps, RadioProps};
pub use select::{Select, SelectOption, SelectProps};
pub use switch::{Switch, SwitchProps};
pub use textarea::{Textarea, TextareaProps};
pub use theme::{ColorScheme, Size, Theme, Variant};
pub use theme_provider::{ThemeMode, ThemeProvider, ThemeProviderProps};
pub use tokens::{
    v, vars, vf, BorderTokens, ColorTokens, DesignTokens, ShadowTokens, SpacingTokens,
    TransitionTokens, TypographyTokens, ZIndexTokens,
};

pub use box_::{Box, BoxProps};
pub use card::{Card, CardBody, CardFooter, CardHeader, CardProps};
pub use container::{Container, ContainerProps, ContainerSize};
pub use divider::{Divider, DividerOrientation, DividerProps};
pub use flex::{AlignItems, Flex, FlexDirection, FlexProps, FlexWrap, JustifyContent};
pub use grid::{Grid, GridProps};
pub use stack::{Stack, StackDirection, StackProps};

pub use alert::{Alert, AlertProps, AlertVariant};
pub use dialog::{Dialog, DialogProps};
pub use popover::{Popover, PopoverProps};
pub use progress::{Progress, ProgressProps};
pub use skeleton::{Skeleton, SkeletonProps, SkeletonShape};
pub use spinner::{Spinner, SpinnerProps};
pub use toast::{Toast, ToastManager, ToastProps, ToastVariant};
pub use tooltip::{Tooltip, TooltipPosition, TooltipProps};

pub use accordion::{Accordion, AccordionItem, AccordionProps};
pub use breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbProps};
pub use link::{Link, LinkProps};
pub use tabs::{TabItem, Tabs, TabsProps};

pub use avatar::{Avatar, AvatarProps};
pub use badge::{Badge, BadgeProps};
pub use list::{List, ListItem, ListProps, ListVariant};
pub use table::{Table, TableColumn, TableProps, TableRow};
pub use tag::{Tag, TagProps};

pub use date_picker::{DatePicker, DatePickerProps};
pub use file_upload::{FileUpload, FileUploadProps};
pub use form_field::{FormField, FormFieldProps};
pub use slider::{Slider, SliderProps};

// Overlay & Interaction
pub mod bottom_sheet;
pub mod command_palette;
pub mod context_menu;
pub mod drawer;
pub mod dropdown;
pub mod hover_card;

// Data Display — advanced
pub mod calendar;
pub mod carousel;
pub mod code_block;
pub mod data_table;
pub mod empty_state;
pub mod stat;
pub mod timeline;
pub mod tree_view;

// Forms — advanced
pub mod autocomplete;
pub mod color_picker;
pub mod form_validator;
pub mod masked_input;
pub mod otp_input;
pub mod rating;
pub mod tag_input;

// Layout — advanced
pub mod aspect_ratio;
pub mod collapsible;
pub mod resizable;
pub mod scroll_area;
pub mod virtual_list;

// Feedback — advanced
pub mod circular_progress;
pub mod confirmation_dialog;
pub mod error_boundary;
pub mod notification;

// Re-exports — Overlay & Interaction
pub use bottom_sheet::{BottomSheet, BottomSheetProps};
pub use command_palette::{CommandItem, CommandPalette, CommandPaletteProps};
pub use context_menu::{ContextMenu, ContextMenuItem, ContextMenuProps};
pub use drawer::{Drawer, DrawerProps, DrawerSide};
pub use dropdown::{Dropdown, DropdownItem, DropdownProps, DropdownSeparator};
pub use hover_card::{HoverCard, HoverCardProps};

// Re-exports — Data Display
pub use calendar::{Calendar, CalendarDay, CalendarProps};
pub use carousel::{Carousel, CarouselProps, CarouselSlide};
pub use code_block::{CodeBlock, CodeBlockProps, CodeLanguage};
pub use data_table::{
    DataColumn, DataRow, DataTable, DataTableProps, FilterConfig, PaginationConfig, SortDirection,
};
pub use empty_state::{EmptyState, EmptyStateProps};
pub use stat::{Stat, StatProps, StatTrend};
pub use timeline::{Timeline, TimelineItem, TimelineProps, TimelineVariant};
pub use tree_view::{TreeNode, TreeView, TreeViewProps};

// Re-exports — Forms advanced
pub use autocomplete::{AutoComplete, AutoCompleteOption, AutoCompleteProps};
pub use color_picker::{ColorPicker, ColorPickerProps};
pub use form_validator::{FieldValidator, FormValidator, ValidationResult, ValidationRule};
pub use masked_input::{MaskPattern, MaskedInput, MaskedInputProps};
pub use otp_input::{OtpInput, OtpInputProps};
pub use rating::{Rating, RatingProps};
pub use tag_input::{TagInput, TagInputProps};

// Re-exports — Layout advanced
pub use aspect_ratio::{AspectRatio, AspectRatioProps};
pub use collapsible::{Collapsible, CollapsibleProps};
pub use resizable::{Resizable, ResizableProps, ResizeDirection};
pub use scroll_area::{ScrollArea, ScrollAreaProps};
pub use virtual_list::{VirtualItem, VirtualList, VirtualListProps};

// Re-exports — Feedback advanced
pub use circular_progress::{CircularProgress, CircularProgressProps};
pub use confirmation_dialog::{ConfirmVariant, ConfirmationDialog, ConfirmationDialogProps};
pub use error_boundary::{ErrorBoundary, ErrorBoundaryProps, ErrorFallback};
pub use notification::{Notification, NotificationProps, NotificationVariant};
