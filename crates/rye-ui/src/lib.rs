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

pub mod tokens;
pub mod theme;
pub mod theme_provider;
pub mod button;
pub mod input;
pub mod textarea;
pub mod select;
pub mod checkbox;
pub mod radio;
pub mod switch;
pub mod label;

pub mod box_;
pub mod flex;
pub mod grid;
pub mod stack;
pub mod card;
pub mod divider;
pub mod container;

pub mod dialog;
pub mod toast;
pub mod tooltip;
pub mod popover;
pub mod alert;
pub mod spinner;
pub mod progress;
pub mod skeleton;

pub mod tabs;
pub mod accordion;
pub mod breadcrumb;
pub mod link;

pub mod table;
pub mod badge;
pub mod avatar;
pub mod tag;
pub mod list;

pub mod form_field;
pub mod slider;
pub mod date_picker;
pub mod file_upload;

// Re-export all components at the crate root for convenience.
pub use tokens::{
    DesignTokens, ColorTokens, TypographyTokens, SpacingTokens,
    BorderTokens, ShadowTokens, ZIndexTokens, TransitionTokens,
    vars, v, vf,
};
pub use theme::{Theme, ColorScheme, Size, Variant};
pub use theme_provider::{ThemeProvider, ThemeProviderProps, ThemeMode};
pub use button::{Button, ButtonProps};
pub use input::{Input, InputProps};
pub use textarea::{Textarea, TextareaProps};
pub use select::{Select, SelectProps, SelectOption};
pub use checkbox::{Checkbox, CheckboxProps};
pub use radio::{Radio, RadioProps, RadioGroup, RadioGroupProps};
pub use switch::{Switch, SwitchProps};
pub use label::{Label, LabelProps};

pub use box_::{Box, BoxProps};
pub use flex::{Flex, FlexProps, FlexDirection, FlexWrap, JustifyContent, AlignItems};
pub use grid::{Grid, GridProps};
pub use stack::{Stack, StackProps, StackDirection};
pub use card::{Card, CardProps, CardHeader, CardBody, CardFooter};
pub use divider::{Divider, DividerProps, DividerOrientation};
pub use container::{Container, ContainerProps, ContainerSize};

pub use dialog::{Dialog, DialogProps};
pub use toast::{Toast, ToastProps, ToastVariant, ToastManager};
pub use tooltip::{Tooltip, TooltipProps, TooltipPosition};
pub use popover::{Popover, PopoverProps};
pub use alert::{Alert, AlertProps, AlertVariant};
pub use spinner::{Spinner, SpinnerProps};
pub use progress::{Progress, ProgressProps};
pub use skeleton::{Skeleton, SkeletonProps, SkeletonShape};

pub use tabs::{Tabs, TabsProps, TabItem};
pub use accordion::{Accordion, AccordionProps, AccordionItem};
pub use breadcrumb::{Breadcrumb, BreadcrumbProps, BreadcrumbItem};
pub use link::{Link, LinkProps};

pub use table::{Table, TableProps, TableColumn, TableRow};
pub use badge::{Badge, BadgeProps};
pub use avatar::{Avatar, AvatarProps};
pub use tag::{Tag, TagProps};
pub use list::{List, ListProps, ListItem, ListVariant};

pub use form_field::{FormField, FormFieldProps};
pub use slider::{Slider, SliderProps};
pub use date_picker::{DatePicker, DatePickerProps};
pub use file_upload::{FileUpload, FileUploadProps};

// Overlay & Interaction
pub mod dropdown;
pub mod context_menu;
pub mod command_palette;
pub mod bottom_sheet;
pub mod drawer;
pub mod hover_card;

// Data Display — advanced
pub mod data_table;
pub mod tree_view;
pub mod timeline;
pub mod calendar;
pub mod carousel;
pub mod code_block;
pub mod empty_state;
pub mod stat;

// Forms — advanced
pub mod color_picker;
pub mod rating;
pub mod otp_input;
pub mod masked_input;
pub mod tag_input;
pub mod autocomplete;
pub mod form_validator;

// Layout — advanced
pub mod aspect_ratio;
pub mod collapsible;
pub mod resizable;
pub mod scroll_area;
pub mod virtual_list;

// Feedback — advanced
pub mod confirmation_dialog;
pub mod circular_progress;
pub mod notification;
pub mod error_boundary;

// Re-exports — Overlay & Interaction
pub use dropdown::{Dropdown, DropdownProps, DropdownItem, DropdownSeparator};
pub use context_menu::{ContextMenu, ContextMenuProps, ContextMenuItem};
pub use command_palette::{CommandPalette, CommandPaletteProps, CommandItem};
pub use bottom_sheet::{BottomSheet, BottomSheetProps};
pub use drawer::{Drawer, DrawerProps, DrawerSide};
pub use hover_card::{HoverCard, HoverCardProps};

// Re-exports — Data Display
pub use data_table::{DataTable, DataTableProps, DataColumn, DataRow, SortDirection, FilterConfig, PaginationConfig};
pub use tree_view::{TreeView, TreeViewProps, TreeNode};
pub use timeline::{Timeline, TimelineProps, TimelineItem, TimelineVariant};
pub use calendar::{Calendar, CalendarProps, CalendarDay};
pub use carousel::{Carousel, CarouselProps, CarouselSlide};
pub use code_block::{CodeBlock, CodeBlockProps, CodeLanguage};
pub use empty_state::{EmptyState, EmptyStateProps};
pub use stat::{Stat, StatProps, StatTrend};

// Re-exports — Forms advanced
pub use color_picker::{ColorPicker, ColorPickerProps};
pub use rating::{Rating, RatingProps};
pub use otp_input::{OtpInput, OtpInputProps};
pub use masked_input::{MaskedInput, MaskedInputProps, MaskPattern};
pub use tag_input::{TagInput, TagInputProps};
pub use autocomplete::{AutoComplete, AutoCompleteProps, AutoCompleteOption};
pub use form_validator::{FormValidator, ValidationRule, ValidationResult, FieldValidator};

// Re-exports — Layout advanced
pub use aspect_ratio::{AspectRatio, AspectRatioProps};
pub use collapsible::{Collapsible, CollapsibleProps};
pub use resizable::{Resizable, ResizableProps, ResizeDirection};
pub use scroll_area::{ScrollArea, ScrollAreaProps};
pub use virtual_list::{VirtualList, VirtualListProps, VirtualItem};

// Re-exports — Feedback advanced
pub use confirmation_dialog::{ConfirmationDialog, ConfirmationDialogProps, ConfirmVariant};
pub use circular_progress::{CircularProgress, CircularProgressProps};
pub use notification::{Notification, NotificationProps, NotificationVariant};
pub use error_boundary::{ErrorBoundary, ErrorBoundaryProps, ErrorFallback};
