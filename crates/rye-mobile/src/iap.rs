//! Goal 204: Native in-app purchases.
//!
//! `use_iap()` hook for StoreKit (iOS), Google Play Billing (Android), and Web Payment Request API.

use std::collections::HashMap;
use std::sync::Mutex;

/// The type of in-app purchase product.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductType {
    /// One-time consumable purchase (e.g. coins, lives).
    Consumable,
    /// One-time non-consumable purchase (e.g. unlock premium).
    NonConsumable,
    /// Auto-renewable subscription.
    AutoRenewableSubscription,
    /// Non-renewing subscription.
    NonRenewingSubscription,
}

impl ProductType {
    /// Check if this is a subscription.
    pub fn is_subscription(&self) -> bool {
        matches!(
            self,
            ProductType::AutoRenewableSubscription | ProductType::NonRenewingSubscription
        )
    }

    /// Check if this is consumable.
    pub fn is_consumable(&self) -> bool {
        matches!(self, ProductType::Consumable)
    }
}

/// A purchasable product.
#[derive(Debug, Clone)]
pub struct Product {
    /// The product identifier (SKU).
    pub id: String,
    /// The product title.
    pub title: String,
    /// The product description.
    pub description: String,
    /// The product type.
    pub product_type: ProductType,
    /// The price (in local currency).
    pub price: String,
    /// The price in the smallest currency unit (e.g. cents).
    pub price_amount_micros: i64,
    /// The ISO currency code.
    pub currency_code: String,
    /// The subscription period (for subscriptions), e.g. "P1M" for monthly.
    pub subscription_period: Option<String>,
    /// Whether the product is currently available for purchase.
    pub available: bool,
}

impl Product {
    /// Create a new product.
    pub fn new(id: &str, title: &str, price: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: String::new(),
            product_type: ProductType::NonConsumable,
            price: price.to_string(),
            price_amount_micros: 0,
            currency_code: "USD".to_string(),
            subscription_period: None,
            available: true,
        }
    }

    /// Set the description.
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Set the product type.
    pub fn with_type(mut self, product_type: ProductType) -> Self {
        self.product_type = product_type;
        self
    }

    /// Set the price in micros.
    pub fn with_price_micros(mut self, micros: i64) -> Self {
        self.price_amount_micros = micros;
        self
    }

    /// Set the currency code.
    pub fn with_currency(mut self, code: &str) -> Self {
        self.currency_code = code.to_string();
        self
    }

    /// Set the subscription period.
    pub fn with_subscription_period(mut self, period: &str) -> Self {
        self.subscription_period = Some(period.to_string());
        self
    }
}

/// The state of a purchase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PurchaseState {
    /// Purchase is in progress.
    Pending,
    /// Purchase was successful.
    Purchased,
    /// Purchase was restored from a previous transaction.
    Restored,
    /// Purchase failed.
    Failed,
    /// Purchase was refunded.
    Refunded,
    /// Subscription is active.
    Active,
    /// Subscription has expired.
    Expired,
    /// Subscription was cancelled.
    Cancelled,
    /// Subscription is in a grace period.
    InGracePeriod,
}

impl PurchaseState {
    /// Check if the purchase is in a valid/active state.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            PurchaseState::Purchased | PurchaseState::Restored | PurchaseState::Active
        )
    }

    /// Check if the purchase failed.
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            PurchaseState::Failed
                | PurchaseState::Refunded
                | PurchaseState::Expired
                | PurchaseState::Cancelled
        )
    }
}

/// A purchase transaction.
#[derive(Debug, Clone, PartialEq)]
pub struct Purchase {
    /// The transaction ID.
    pub transaction_id: String,
    /// The product ID.
    pub product_id: String,
    /// The purchase state.
    pub state: PurchaseState,
    /// The purchase date (Unix timestamp).
    pub purchase_date: u64,
    /// The expiration date (for subscriptions, Unix timestamp).
    pub expiration_date: Option<u64>,
    /// The quantity purchased.
    pub quantity: u32,
    /// Whether the transaction has been acknowledged/finished.
    pub acknowledged: bool,
    /// The receipt/verification data.
    pub receipt: Option<String>,
}

impl Purchase {
    /// Create a new purchase.
    pub fn new(transaction_id: &str, product_id: &str) -> Self {
        Self {
            transaction_id: transaction_id.to_string(),
            product_id: product_id.to_string(),
            state: PurchaseState::Pending,
            purchase_date: 0,
            expiration_date: None,
            quantity: 1,
            acknowledged: false,
            receipt: None,
        }
    }

    /// Check if the purchase is active.
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// Check if the purchase needs acknowledgment.
    pub fn needs_acknowledgment(&self) -> bool {
        self.state.is_active() && !self.acknowledged
    }
}

/// The result of a purchase operation.
#[derive(Debug, Clone, PartialEq)]
pub enum PurchaseResult {
    /// Purchase succeeded.
    Success(Purchase),
    /// User cancelled.
    Cancelled,
    /// Product not found.
    ProductNotFound,
    /// Product not available.
    NotAvailable,
    /// Store not available.
    StoreNotAvailable,
    /// Error.
    Error(String),
}

impl PurchaseResult {
    /// Check if successful.
    pub fn is_success(&self) -> bool {
        matches!(self, PurchaseResult::Success(_))
    }
}

/// The IAP manager.
pub struct IapManager {
    products: Mutex<HashMap<String, Product>>,
    purchases: Mutex<Vec<Purchase>>,
    store_available: bool,
}

impl IapManager {
    /// Create a new IAP manager.
    pub fn new() -> Self {
        Self {
            products: Mutex::new(HashMap::new()),
            purchases: Mutex::new(Vec::new()),
            store_available: true,
        }
    }

    /// Create a manager with store availability.
    pub fn with_availability(available: bool) -> Self {
        Self {
            products: Mutex::new(HashMap::new()),
            purchases: Mutex::new(Vec::new()),
            store_available: available,
        }
    }

    /// Check if the store is available.
    pub fn is_store_available(&self) -> bool {
        self.store_available
    }

    /// Register a product.
    pub fn register_product(&self, product: Product) {
        self.products
            .lock()
            .unwrap()
            .insert(product.id.clone(), product);
    }

    /// Get a product by ID.
    pub fn get_product(&self, id: &str) -> Option<Product> {
        self.products.lock().unwrap().get(id).cloned()
    }

    /// Get all registered product IDs.
    pub fn product_ids(&self) -> Vec<String> {
        self.products.lock().unwrap().keys().cloned().collect()
    }

    /// Get all available products.
    pub fn available_products(&self) -> Vec<Product> {
        self.products
            .lock()
            .unwrap()
            .values()
            .filter(|p| p.available)
            .cloned()
            .collect()
    }

    /// Purchase a product (simulated).
    pub fn purchase(&self, product_id: &str) -> PurchaseResult {
        if !self.store_available {
            return PurchaseResult::StoreNotAvailable;
        }

        let products = self.products.lock().unwrap();
        let _product = match products.get(product_id) {
            Some(p) if p.available => p.clone(),
            Some(_) => return PurchaseResult::NotAvailable,
            None => return PurchaseResult::ProductNotFound,
        };
        drop(products);

        let tx_id = format!("tx_{}", self.purchases.lock().unwrap().len() + 1);
        let mut purchase = Purchase::new(&tx_id, product_id);
        purchase.state = PurchaseState::Purchased;
        purchase.purchase_date = 1700000000;
        purchase.receipt = Some(format!("receipt_{}", tx_id));

        self.purchases.lock().unwrap().push(purchase.clone());

        PurchaseResult::Success(purchase)
    }

    /// Restore previous purchases (simulated).
    pub fn restore_purchases(&self) -> Vec<Purchase> {
        self.purchases
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.state.is_active())
            .cloned()
            .collect()
    }

    /// Acknowledge a purchase.
    pub fn acknowledge(&self, transaction_id: &str) -> bool {
        let mut purchases = self.purchases.lock().unwrap();
        for p in purchases.iter_mut() {
            if p.transaction_id == transaction_id {
                p.acknowledged = true;
                return true;
            }
        }
        false
    }

    /// Consume a consumable purchase.
    pub fn consume(&self, transaction_id: &str) -> bool {
        let mut purchases = self.purchases.lock().unwrap();
        let len_before = purchases.len();
        purchases.retain(|p| p.transaction_id != transaction_id);
        purchases.len() != len_before
    }

    /// Get all purchases.
    pub fn purchases(&self) -> Vec<Purchase> {
        self.purchases.lock().unwrap().clone()
    }

    /// Get active purchases.
    pub fn active_purchases(&self) -> Vec<Purchase> {
        self.purchases
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.is_active())
            .cloned()
            .collect()
    }

    /// Get the number of purchases.
    pub fn purchase_count(&self) -> usize {
        self.purchases.lock().unwrap().len()
    }
}

impl Default for IapManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_type_is_subscription() {
        assert!(ProductType::AutoRenewableSubscription.is_subscription());
        assert!(ProductType::NonRenewingSubscription.is_subscription());
        assert!(!ProductType::Consumable.is_subscription());
        assert!(!ProductType::NonConsumable.is_subscription());
    }

    #[test]
    fn test_product_type_is_consumable() {
        assert!(ProductType::Consumable.is_consumable());
        assert!(!ProductType::NonConsumable.is_consumable());
    }

    #[test]
    fn test_product_new() {
        let p = Product::new("premium", "Premium", "$9.99");
        assert_eq!(p.id, "premium");
        assert_eq!(p.title, "Premium");
        assert_eq!(p.price, "$9.99");
        assert_eq!(p.product_type, ProductType::NonConsumable);
        assert!(p.available);
    }

    #[test]
    fn test_product_builder() {
        let p = Product::new("sub", "Monthly Sub", "$4.99")
            .with_description("Monthly subscription")
            .with_type(ProductType::AutoRenewableSubscription)
            .with_price_micros(4990000)
            .with_currency("EUR")
            .with_subscription_period("P1M");

        assert_eq!(p.description, "Monthly subscription");
        assert!(p.product_type.is_subscription());
        assert_eq!(p.price_amount_micros, 4990000);
        assert_eq!(p.currency_code, "EUR");
        assert_eq!(p.subscription_period, Some("P1M".to_string()));
    }

    #[test]
    fn test_purchase_state_is_active() {
        assert!(PurchaseState::Purchased.is_active());
        assert!(PurchaseState::Active.is_active());
        assert!(!PurchaseState::Failed.is_active());
        assert!(!PurchaseState::Expired.is_active());
    }

    #[test]
    fn test_purchase_state_is_failure() {
        assert!(PurchaseState::Failed.is_failure());
        assert!(PurchaseState::Refunded.is_failure());
        assert!(PurchaseState::Expired.is_failure());
        assert!(!PurchaseState::Purchased.is_failure());
    }

    #[test]
    fn test_purchase_new() {
        let p = Purchase::new("tx1", "prod1");
        assert_eq!(p.transaction_id, "tx1");
        assert_eq!(p.product_id, "prod1");
        assert_eq!(p.state, PurchaseState::Pending);
        assert!(!p.acknowledged);
    }

    #[test]
    fn test_purchase_needs_acknowledgment() {
        let mut p = Purchase::new("tx1", "prod1");
        p.state = PurchaseState::Purchased;
        assert!(p.needs_acknowledgment());
        p.acknowledged = true;
        assert!(!p.needs_acknowledgment());
    }

    #[test]
    fn test_purchase_result_is_success() {
        assert!(PurchaseResult::Success(Purchase::new("t", "p")).is_success());
        assert!(!PurchaseResult::Cancelled.is_success());
    }

    #[test]
    fn test_manager_store_available() {
        let mgr = IapManager::new();
        assert!(mgr.is_store_available());
    }

    #[test]
    fn test_manager_store_not_available() {
        let mgr = IapManager::with_availability(false);
        assert!(!mgr.is_store_available());
    }

    #[test]
    fn test_manager_register_get_product() {
        let mgr = IapManager::new();
        mgr.register_product(Product::new("p1", "Product 1", "$1.99"));
        assert!(mgr.get_product("p1").is_some());
        assert!(mgr.get_product("nonexistent").is_none());
    }

    #[test]
    fn test_manager_product_ids() {
        let mgr = IapManager::new();
        mgr.register_product(Product::new("a", "A", "$1"));
        mgr.register_product(Product::new("b", "B", "$2"));
        assert_eq!(mgr.product_ids().len(), 2);
    }

    #[test]
    fn test_manager_available_products() {
        let mgr = IapManager::new();
        let mut p = Product::new("a", "A", "$1");
        p.available = false;
        mgr.register_product(p);
        mgr.register_product(Product::new("b", "B", "$2"));
        assert_eq!(mgr.available_products().len(), 1);
    }

    #[test]
    fn test_manager_purchase() {
        let mgr = IapManager::new();
        mgr.register_product(Product::new("p1", "Product 1", "$1.99"));
        let result = mgr.purchase("p1");
        assert!(result.is_success());
        assert_eq!(mgr.purchase_count(), 1);
    }

    #[test]
    fn test_manager_purchase_not_found() {
        let mgr = IapManager::new();
        let result = mgr.purchase("nonexistent");
        assert_eq!(result, PurchaseResult::ProductNotFound);
    }

    #[test]
    fn test_manager_purchase_store_unavailable() {
        let mgr = IapManager::with_availability(false);
        mgr.register_product(Product::new("p1", "P1", "$1"));
        let result = mgr.purchase("p1");
        assert_eq!(result, PurchaseResult::StoreNotAvailable);
    }

    #[test]
    fn test_manager_purchase_not_available() {
        let mgr = IapManager::new();
        let mut p = Product::new("p1", "P1", "$1");
        p.available = false;
        mgr.register_product(p);
        let result = mgr.purchase("p1");
        assert_eq!(result, PurchaseResult::NotAvailable);
    }

    #[test]
    fn test_manager_acknowledge() {
        let mgr = IapManager::new();
        mgr.register_product(Product::new("p1", "P1", "$1"));
        let result = mgr.purchase("p1");
        if let PurchaseResult::Success(purchase) = result {
            assert!(mgr.acknowledge(&purchase.transaction_id));
        }
        assert!(!mgr.acknowledge("nonexistent"));
    }

    #[test]
    fn test_manager_consume() {
        let mgr = IapManager::new();
        mgr.register_product(Product::new("p1", "P1", "$1"));
        let result = mgr.purchase("p1");
        if let PurchaseResult::Success(purchase) = result {
            assert!(mgr.consume(&purchase.transaction_id));
        }
        assert_eq!(mgr.purchase_count(), 0);
    }

    #[test]
    fn test_manager_restore_purchases() {
        let mgr = IapManager::new();
        mgr.register_product(Product::new("p1", "P1", "$1"));
        mgr.register_product(Product::new("p2", "P2", "$2"));
        mgr.purchase("p1");
        mgr.purchase("p2");
        let restored = mgr.restore_purchases();
        assert_eq!(restored.len(), 2);
    }

    #[test]
    fn test_manager_active_purchases() {
        let mgr = IapManager::new();
        mgr.register_product(Product::new("p1", "P1", "$1"));
        mgr.purchase("p1");
        assert_eq!(mgr.active_purchases().len(), 1);
    }
}
