//! Goal 202: Native contacts access.
//!
//! `use_contacts()` hook for reading the device's contact list.

use std::sync::Mutex;

/// A contact from the device's address book.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Contact {
    /// The unique contact identifier.
    pub id: String,
    /// The display name.
    pub display_name: String,
    /// Given / first name.
    pub given_name: String,
    /// Family / last name.
    pub family_name: String,
    /// Middle name.
    pub middle_name: String,
    /// Phone numbers (label -> number).
    pub phone_numbers: Vec<ContactField>,
    /// Email addresses (label -> email).
    pub emails: Vec<ContactField>,
    /// Postal addresses.
    pub postal_addresses: Vec<ContactAddress>,
    /// Organization / company.
    pub organization: Option<String>,
    /// Job title.
    pub job_title: Option<String>,
    /// Birthday (ISO date string).
    pub birthday: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// Avatar / photo (base64 or file path).
    pub avatar: Option<String>,
}

impl Contact {
    /// Create a new contact with an ID and display name.
    pub fn new(id: &str, display_name: &str) -> Self {
        Self {
            id: id.to_string(),
            display_name: display_name.to_string(),
            ..Default::default()
        }
    }

    /// Set given and family name.
    pub fn with_name(mut self, given: &str, family: &str) -> Self {
        self.given_name = given.to_string();
        self.family_name = family.to_string();
        self
    }

    /// Add a phone number.
    pub fn add_phone(mut self, label: &str, number: &str) -> Self {
        self.phone_numbers.push(ContactField {
            label: label.to_string(),
            value: number.to_string(),
        });
        self
    }

    /// Add an email.
    pub fn add_email(mut self, label: &str, email: &str) -> Self {
        self.emails.push(ContactField {
            label: label.to_string(),
            value: email.to_string(),
        });
        self
    }

    /// Set organization.
    pub fn with_organization(mut self, org: &str) -> Self {
        self.organization = Some(org.to_string());
        self
    }

    /// Set avatar.
    pub fn with_avatar(mut self, avatar: &str) -> Self {
        self.avatar = Some(avatar.to_string());
        self
    }

    /// Get the primary phone number (first one).
    pub fn primary_phone(&self) -> Option<&str> {
        self.phone_numbers.first().map(|f| f.value.as_str())
    }

    /// Get the primary email (first one).
    pub fn primary_email(&self) -> Option<&str> {
        self.emails.first().map(|f| f.value.as_str())
    }

    /// Get the full name.
    pub fn full_name(&self) -> String {
        if !self.given_name.is_empty() || !self.family_name.is_empty() {
            format!("{} {}", self.given_name, self.family_name).trim().to_string()
        } else {
            self.display_name.clone()
        }
    }
}

/// A labeled contact field (phone, email, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct ContactField {
    /// The field label (e.g. "home", "work", "mobile").
    pub label: String,
    /// The field value.
    pub value: String,
}

/// A postal address.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContactAddress {
    /// The address label (e.g. "home", "work").
    pub label: String,
    /// Street address.
    pub street: String,
    /// City.
    pub city: String,
    /// State / province / region.
    pub state: String,
    /// Postal / ZIP code.
    pub postal_code: String,
    /// Country.
    pub country: String,
}

impl ContactAddress {
    /// Create a new address.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            ..Default::default()
        }
    }

    /// Format as a single string.
    pub fn formatted(&self) -> String {
        let parts: Vec<&str> = [
            self.street.as_str(),
            self.city.as_str(),
            self.state.as_str(),
            self.postal_code.as_str(),
            self.country.as_str(),
        ]
        .iter()
        .filter(|s| !s.is_empty())
        .copied()
        .collect();
        parts.join(", ")
    }
}

/// Configuration for contact fetching.
#[derive(Debug, Clone, Default)]
pub struct ContactsConfig {
    /// Whether to fetch phone numbers.
    pub fetch_phone_numbers: bool,
    /// Whether to fetch emails.
    pub fetch_emails: bool,
    /// Whether to fetch postal addresses.
    pub fetch_addresses: bool,
    /// Whether to fetch avatars/photos.
    pub fetch_avatars: bool,
    /// Whether to fetch organizations.
    pub fetch_organizations: bool,
    /// Maximum number of contacts to return (None = all).
    pub limit: Option<usize>,
    /// Search query to filter contacts.
    pub search_query: Option<String>,
}

impl ContactsConfig {
    /// Create a config that fetches everything.
    pub fn all() -> Self {
        Self {
            fetch_phone_numbers: true,
            fetch_emails: true,
            fetch_addresses: true,
            fetch_avatars: true,
            fetch_organizations: true,
            limit: None,
            search_query: None,
        }
    }

    /// Create a config that fetches only phone numbers.
    pub fn phones_only() -> Self {
        Self {
            fetch_phone_numbers: true,
            ..Default::default()
        }
    }

    /// Set a limit.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set a search query.
    pub fn with_search(mut self, query: &str) -> Self {
        self.search_query = Some(query.to_string());
        self
    }
}

/// The result of a contacts fetch.
#[derive(Debug, Clone, PartialEq)]
pub enum ContactsResult {
    /// Success with the fetched contacts.
    Success(Vec<Contact>),
    /// Permission denied.
    PermissionDenied,
    /// Contacts not available.
    NotAvailable,
    /// Error.
    Error(String),
}

impl ContactsResult {
    /// Check if successful.
    pub fn is_success(&self) -> bool {
        matches!(self, ContactsResult::Success(_))
    }
}

/// The contacts manager.
pub struct ContactsManager {
    has_permission: Mutex<bool>,
    contacts: Mutex<Vec<Contact>>,
}

impl ContactsManager {
    /// Create a new contacts manager.
    pub fn new() -> Self {
        Self {
            has_permission: Mutex::new(false),
            contacts: Mutex::new(Vec::new()),
        }
    }

    /// Request contacts permission.
    pub fn request_permission(&self) -> bool {
        *self.has_permission.lock().unwrap() = true;
        true
    }

    /// Check if permission is granted.
    pub fn has_permission(&self) -> bool {
        *self.has_permission.lock().unwrap()
    }

    /// Add a contact to the internal store (for testing).
    pub fn add_contact(&self, contact: Contact) {
        self.contacts.lock().unwrap().push(contact);
    }

    /// Fetch contacts (simulated, reads from internal store).
    pub fn fetch(&self, config: &ContactsConfig) -> ContactsResult {
        if !*self.has_permission.lock().unwrap() {
            return ContactsResult::PermissionDenied;
        }

        let contacts = self.contacts.lock().unwrap();
        let mut filtered: Vec<Contact> = contacts
            .iter()
            .filter(|c| {
                if let Some(ref query) = config.search_query {
                    let q = query.to_lowercase();
                    c.display_name.to_lowercase().contains(&q)
                        || c.given_name.to_lowercase().contains(&q)
                        || c.family_name.to_lowercase().contains(&q)
                        || c.phone_numbers.iter().any(|p| p.value.contains(&q))
                        || c.emails.iter().any(|e| e.value.to_lowercase().contains(&q))
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        if let Some(limit) = config.limit {
            filtered.truncate(limit);
        }

        ContactsResult::Success(filtered)
    }

    /// Get a contact by ID.
    pub fn get_by_id(&self, id: &str) -> Option<Contact> {
        self.contacts.lock().unwrap().iter().find(|c| c.id == id).cloned()
    }

    /// Get the total number of contacts.
    pub fn count(&self) -> usize {
        self.contacts.lock().unwrap().len()
    }
}

impl Default for ContactsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_new() {
        let c = Contact::new("1", "Alice");
        assert_eq!(c.id, "1");
        assert_eq!(c.display_name, "Alice");
    }

    #[test]
    fn test_contact_builder() {
        let c = Contact::new("1", "Alice Smith")
            .with_name("Alice", "Smith")
            .add_phone("mobile", "555-1234")
            .add_email("home", "alice@example.com")
            .with_organization("ACME Corp")
            .with_avatar("/avatars/alice.png");

        assert_eq!(c.given_name, "Alice");
        assert_eq!(c.family_name, "Smith");
        assert_eq!(c.phone_numbers.len(), 1);
        assert_eq!(c.emails.len(), 1);
        assert_eq!(c.organization, Some("ACME Corp".to_string()));
        assert_eq!(c.avatar, Some("/avatars/alice.png".to_string()));
    }

    #[test]
    fn test_contact_primary_phone() {
        let c = Contact::new("1", "Alice")
            .add_phone("mobile", "555-1234")
            .add_phone("work", "555-5678");
        assert_eq!(c.primary_phone(), Some("555-1234"));
    }

    #[test]
    fn test_contact_primary_email_none() {
        let c = Contact::new("1", "Alice");
        assert!(c.primary_email().is_none());
    }

    #[test]
    fn test_contact_full_name_with_given_family() {
        let c = Contact::new("1", "Alice Smith").with_name("Alice", "Smith");
        assert_eq!(c.full_name(), "Alice Smith");
    }

    #[test]
    fn test_contact_full_name_fallback_display() {
        let c = Contact::new("1", "Bob");
        assert_eq!(c.full_name(), "Bob");
    }

    #[test]
    fn test_contact_address_formatted() {
        let addr = ContactAddress {
            label: "home".to_string(),
            street: "123 Main St".to_string(),
            city: "SF".to_string(),
            state: "CA".to_string(),
            postal_code: "94101".to_string(),
            country: "USA".to_string(),
        };
        assert_eq!(addr.formatted(), "123 Main St, SF, CA, 94101, USA");
    }

    #[test]
    fn test_contact_address_formatted_partial() {
        let addr = ContactAddress {
            label: "work".to_string(),
            street: String::new(),
            city: "SF".to_string(),
            state: String::new(),
            postal_code: String::new(),
            country: "USA".to_string(),
        };
        assert_eq!(addr.formatted(), "SF, USA");
    }

    #[test]
    fn test_contacts_config_all() {
        let config = ContactsConfig::all();
        assert!(config.fetch_phone_numbers);
        assert!(config.fetch_emails);
        assert!(config.fetch_addresses);
        assert!(config.fetch_avatars);
        assert!(config.fetch_organizations);
    }

    #[test]
    fn test_contacts_config_phones_only() {
        let config = ContactsConfig::phones_only();
        assert!(config.fetch_phone_numbers);
        assert!(!config.fetch_emails);
    }

    #[test]
    fn test_contacts_config_with_limit_search() {
        let config = ContactsConfig::all().with_limit(10).with_search("Alice");
        assert_eq!(config.limit, Some(10));
        assert_eq!(config.search_query, Some("Alice".to_string()));
    }

    #[test]
    fn test_contacts_result_is_success() {
        assert!(ContactsResult::Success(vec![]).is_success());
        assert!(!ContactsResult::PermissionDenied.is_success());
    }

    #[test]
    fn test_manager_permission() {
        let mgr = ContactsManager::new();
        assert!(!mgr.has_permission());
        mgr.request_permission();
        assert!(mgr.has_permission());
    }

    #[test]
    fn test_manager_fetch_no_permission() {
        let mgr = ContactsManager::new();
        let result = mgr.fetch(&ContactsConfig::all());
        assert_eq!(result, ContactsResult::PermissionDenied);
    }

    #[test]
    fn test_manager_fetch() {
        let mgr = ContactsManager::new();
        mgr.request_permission();
        mgr.add_contact(Contact::new("1", "Alice").add_phone("mobile", "555-1234"));
        mgr.add_contact(Contact::new("2", "Bob").add_email("home", "bob@test.com"));

        let result = mgr.fetch(&ContactsConfig::all());
        assert!(result.is_success());
        if let ContactsResult::Success(contacts) = result {
            assert_eq!(contacts.len(), 2);
        }
    }

    #[test]
    fn test_manager_fetch_with_limit() {
        let mgr = ContactsManager::new();
        mgr.request_permission();
        mgr.add_contact(Contact::new("1", "Alice"));
        mgr.add_contact(Contact::new("2", "Bob"));
        mgr.add_contact(Contact::new("3", "Carol"));

        let result = mgr.fetch(&ContactsConfig::all().with_limit(2));
        if let ContactsResult::Success(contacts) = result {
            assert_eq!(contacts.len(), 2);
        }
    }

    #[test]
    fn test_manager_fetch_with_search() {
        let mgr = ContactsManager::new();
        mgr.request_permission();
        mgr.add_contact(Contact::new("1", "Alice Smith"));
        mgr.add_contact(Contact::new("2", "Bob Jones"));

        let result = mgr.fetch(&ContactsConfig::all().with_search("alice"));
        if let ContactsResult::Success(contacts) = result {
            assert_eq!(contacts.len(), 1);
            assert_eq!(contacts[0].display_name, "Alice Smith");
        }
    }

    #[test]
    fn test_manager_get_by_id() {
        let mgr = ContactsManager::new();
        mgr.add_contact(Contact::new("42", "Alice"));
        assert!(mgr.get_by_id("42").is_some());
        assert!(mgr.get_by_id("99").is_none());
    }

    #[test]
    fn test_manager_count() {
        let mgr = ContactsManager::new();
        mgr.add_contact(Contact::new("1", "Alice"));
        mgr.add_contact(Contact::new("2", "Bob"));
        assert_eq!(mgr.count(), 2);
    }
}
